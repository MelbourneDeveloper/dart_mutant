//! Mutation test runner with parallel execution
//!
//! This module handles running tests against mutated code and collecting results.

pub use crate::mutation::{MutantStatus, Mutation};
use anyhow::{Context, Result};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;

/// RAII guard that restores a file to its original content on drop
struct FileRestoreGuard {
    path: PathBuf,
    original_content: String,
}

impl Drop for FileRestoreGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::write(&self.path, &self.original_content) {
            eprintln!("Warning: Failed to restore file {:?}: {}", self.path, e);
        }
    }
}

/// Result of testing a single mutation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutantTestResult {
    pub mutation: Mutation,
    pub status: MutantStatus,
    pub duration: Duration,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Type alias for per-file locks to prevent concurrent mutations on same file
type FileLocks = Arc<Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>>;

/// Get or create a lock for a specific file
async fn get_file_lock(file_locks: &FileLocks, file_path: &Path) -> Arc<Mutex<()>> {
    let mut locks = file_locks.lock().await;
    locks
        .entry(file_path.to_path_buf())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// Run mutation tests in parallel
///
/// Mutations are run in parallel, but mutations targeting the same file
/// are serialized to prevent race conditions where one mutation overwrites
/// another's changes.
pub async fn run_mutation_tests(
    project_path: &Path,
    mutations: &[Mutation],
    parallel_jobs: usize,
    timeout_secs: u64,
    progress: ProgressBar,
) -> Result<Vec<MutantTestResult>> {
    let semaphore = Arc::new(Semaphore::new(parallel_jobs));
    let project_path = Arc::new(project_path.to_path_buf());
    let timeout_duration = Duration::from_secs(timeout_secs);

    // Per-file locks to prevent concurrent mutations on the same file
    let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));

    // Counters for progress display
    let killed = Arc::new(AtomicUsize::new(0));
    let survived = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = mutations
        .iter()
        .cloned()
        .map(|mutation| {
            let semaphore = semaphore.clone();
            let project_path = project_path.clone();
            let progress = progress.clone();
            let killed = killed.clone();
            let survived = survived.clone();
            let file_locks = file_locks.clone();

            tokio::spawn(async move {
                let Ok(_permit) = semaphore.acquire().await else {
                    return MutantTestResult {
                        mutation: mutation.clone(),
                        status: MutantStatus::Error,
                        duration: Duration::ZERO,
                        output: None,
                        error: Some("Failed to acquire semaphore".to_owned()),
                    };
                };

                // Acquire per-file lock to prevent concurrent mutations on same file
                let file_lock = get_file_lock(&file_locks, &mutation.location.file).await;
                let _file_guard = file_lock.lock().await;

                let result = test_single_mutation(&project_path, &mutation, timeout_duration).await;

                // Update counters and progress
                match result.status {
                    MutantStatus::Killed | MutantStatus::Timeout => {
                        killed.fetch_add(1, Ordering::SeqCst);
                    }
                    MutantStatus::Survived => {
                        survived.fetch_add(1, Ordering::SeqCst);
                    }
                    _ => {}
                }

                let k = killed.load(Ordering::SeqCst);
                let s = survived.load(Ordering::SeqCst);
                progress.set_message(format!("killed: {} survived: {}", k, s));
                progress.inc(1);

                result
            })
        })
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await?);
    }

    Ok(results)
}

/// Test a single mutation
async fn test_single_mutation(
    project_path: &Path,
    mutation: &Mutation,
    timeout_duration: Duration,
) -> MutantTestResult {
    let start = Instant::now();

    // Read the original file
    let file_path = &mutation.location.file;
    let original_source = match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            return MutantTestResult {
                mutation: mutation.clone(),
                status: MutantStatus::Error,
                duration: start.elapsed(),
                output: None,
                error: Some(format!("Failed to read file: {}", e)),
            };
        }
    };

    // Apply the mutation
    let mutated_source = mutation.apply(&original_source);

    // Create RAII guard to restore file on any exit path (including panic)
    let _restore_guard = FileRestoreGuard {
        path: file_path.clone(),
        original_content: original_source,
    };

    // Write the mutated file
    if let Err(e) = std::fs::write(file_path, &mutated_source) {
        return MutantTestResult {
            mutation: mutation.clone(),
            status: MutantStatus::Error,
            duration: start.elapsed(),
            output: None,
            error: Some(format!("Failed to write mutated file: {}", e)),
        };
    }

    // Run the test command
    let test_result = timeout(timeout_duration, run_dart_test(project_path)).await;

    // File will be restored by _restore_guard when it goes out of scope

    // Interpret the result
    let (status, output, error) = match test_result {
        Ok(Ok((exit_code, stdout, stderr))) => {
            if exit_code == 0 {
                // Tests passed - mutation survived (bad!)
                (MutantStatus::Survived, Some(stdout), None)
            } else {
                // Tests failed - mutation killed (good!)
                (MutantStatus::Killed, Some(stdout), Some(stderr))
            }
        }
        Ok(Err(e)) => (MutantStatus::Error, None, Some(e.to_string())),
        Err(_) => {
            // Timeout - counts as killed (infinite loop protection)
            (
                MutantStatus::Timeout,
                None,
                Some("Test timed out".to_string()),
            )
        }
    };

    MutantTestResult {
        mutation: mutation.clone(),
        status,
        duration: start.elapsed(),
        output,
        error,
    }
}

/// Run `dart test` and return (exit_code, stdout, stderr)
async fn run_dart_test(project_path: &Path) -> Result<(i32, String, String)> {
    let output = Command::new("dart")
        .arg("test")
        .arg("--reporter=compact")
        .current_dir(project_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to run dart test")?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((exit_code, stdout, stderr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutation::{MutationOperator, SourceLocation};
    use std::path::PathBuf;
    use std::sync::atomic::AtomicU32;

    fn create_test_mutation() -> Mutation {
        Mutation {
            id: "test".to_string(),
            location: SourceLocation {
                file: PathBuf::from("/tmp/test.dart"),
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 2,
                byte_start: 0,
                byte_end: 1,
            },
            operator: MutationOperator::Arithmetic,
            original: "+".to_string(),
            mutated: "-".to_string(),
            description: "test".to_string(),
            replacements: vec!["-".to_string()],
            ai_suggested: false,
            ai_confidence: None,
        }
    }

    #[allow(dead_code)]
    fn create_mutation_for_file(file: &Path, id: &str) -> Mutation {
        Mutation {
            id: id.to_string(),
            location: SourceLocation {
                file: file.to_path_buf(),
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 2,
                byte_start: 0,
                byte_end: 1,
            },
            operator: MutationOperator::Arithmetic,
            original: "+".to_string(),
            mutated: "-".to_string(),
            description: format!("mutation {}", id),
            replacements: vec!["-".to_string()],
            ai_suggested: false,
            ai_confidence: None,
        }
    }

    #[test]
    fn test_mutation_creation() {
        let mutation = create_test_mutation();
        assert_eq!(mutation.id, "test");
        assert_eq!(mutation.original, "+");
        assert_eq!(mutation.mutated, "-");
    }

    #[tokio::test]
    async fn test_file_lock_creation() {
        let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));
        let path = PathBuf::from("/tmp/test_file.dart");

        // Get lock for a file
        let lock1 = get_file_lock(&file_locks, &path).await;

        // Same file should return same lock
        let lock2 = get_file_lock(&file_locks, &path).await;

        // They should be the same Arc (same memory address)
        assert!(Arc::ptr_eq(&lock1, &lock2));
    }

    #[tokio::test]
    async fn test_different_files_get_different_locks() {
        let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));
        let path1 = PathBuf::from("/tmp/file1.dart");
        let path2 = PathBuf::from("/tmp/file2.dart");

        let lock1 = get_file_lock(&file_locks, &path1).await;
        let lock2 = get_file_lock(&file_locks, &path2).await;

        // Different files should have different locks
        assert!(!Arc::ptr_eq(&lock1, &lock2));
    }

    #[tokio::test]
    async fn test_file_lock_prevents_concurrent_access() {
        let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));
        let path = PathBuf::from("/tmp/concurrent_test.dart");

        // Counter to track concurrent access
        let concurrent_count = Arc::new(AtomicU32::new(0));
        let max_concurrent = Arc::new(AtomicU32::new(0));

        let mut handles = Vec::new();

        // Spawn 10 tasks trying to access the same file
        for i in 0..10 {
            let file_locks = file_locks.clone();
            let path = path.clone();
            let concurrent_count = concurrent_count.clone();
            let max_concurrent = max_concurrent.clone();

            handles.push(tokio::spawn(async move {
                let file_lock = get_file_lock(&file_locks, &path).await;
                let _guard = file_lock.lock().await;

                // Increment counter (we're now accessing the "file")
                let current = concurrent_count.fetch_add(1, Ordering::SeqCst) + 1;

                // Track maximum concurrent access
                max_concurrent.fetch_max(current, Ordering::SeqCst);

                // Simulate some work
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Decrement counter
                concurrent_count.fetch_sub(1, Ordering::SeqCst);

                i
            }));
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Maximum concurrent access should be 1 (serialized)
        assert_eq!(
            max_concurrent.load(Ordering::SeqCst),
            1,
            "File lock should prevent concurrent access - max concurrent was {}",
            max_concurrent.load(Ordering::SeqCst)
        );
    }

    #[tokio::test]
    async fn test_different_files_can_be_accessed_concurrently() {
        let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));

        // Counter to track concurrent access
        let concurrent_count = Arc::new(AtomicU32::new(0));
        let max_concurrent = Arc::new(AtomicU32::new(0));

        let mut handles = Vec::new();

        // Spawn 10 tasks accessing DIFFERENT files
        for i in 0..10 {
            let file_locks = file_locks.clone();
            let path = PathBuf::from(format!("/tmp/file_{}.dart", i));
            let concurrent_count = concurrent_count.clone();
            let max_concurrent = max_concurrent.clone();

            handles.push(tokio::spawn(async move {
                let file_lock = get_file_lock(&file_locks, &path).await;
                let _guard = file_lock.lock().await;

                // Increment counter
                let current = concurrent_count.fetch_add(1, Ordering::SeqCst) + 1;

                // Track maximum concurrent access
                max_concurrent.fetch_max(current, Ordering::SeqCst);

                // Simulate some work - long enough to overlap
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Decrement counter
                concurrent_count.fetch_sub(1, Ordering::SeqCst);

                i
            }));
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Multiple files should be accessed concurrently
        assert!(
            max_concurrent.load(Ordering::SeqCst) > 1,
            "Different files should allow concurrent access - max concurrent was {}",
            max_concurrent.load(Ordering::SeqCst)
        );
    }

    #[tokio::test]
    async fn test_mixed_file_access_pattern() {
        // Test a realistic pattern: some mutations on same file, some on different files
        let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));

        let file_a = PathBuf::from("/tmp/file_a.dart");
        let file_b = PathBuf::from("/tmp/file_b.dart");

        let file_a_concurrent = Arc::new(AtomicU32::new(0));
        let file_a_max = Arc::new(AtomicU32::new(0));
        let file_b_concurrent = Arc::new(AtomicU32::new(0));
        let file_b_max = Arc::new(AtomicU32::new(0));

        let mut handles = Vec::new();

        // 5 tasks for file A
        for i in 0..5 {
            let file_locks = file_locks.clone();
            let path = file_a.clone();
            let concurrent = file_a_concurrent.clone();
            let max = file_a_max.clone();

            handles.push(tokio::spawn(async move {
                let file_lock = get_file_lock(&file_locks, &path).await;
                let _guard = file_lock.lock().await;

                let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                max.fetch_max(current, Ordering::SeqCst);

                tokio::time::sleep(Duration::from_millis(20)).await;

                concurrent.fetch_sub(1, Ordering::SeqCst);
                format!("A-{}", i)
            }));
        }

        // 5 tasks for file B
        for i in 0..5 {
            let file_locks = file_locks.clone();
            let path = file_b.clone();
            let concurrent = file_b_concurrent.clone();
            let max = file_b_max.clone();

            handles.push(tokio::spawn(async move {
                let file_lock = get_file_lock(&file_locks, &path).await;
                let _guard = file_lock.lock().await;

                let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                max.fetch_max(current, Ordering::SeqCst);

                tokio::time::sleep(Duration::from_millis(20)).await;

                concurrent.fetch_sub(1, Ordering::SeqCst);
                format!("B-{}", i)
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Each file should only have 1 concurrent access
        assert_eq!(
            file_a_max.load(Ordering::SeqCst),
            1,
            "File A max concurrent should be 1"
        );
        assert_eq!(
            file_b_max.load(Ordering::SeqCst),
            1,
            "File B max concurrent should be 1"
        );
    }

    #[tokio::test]
    async fn test_lock_released_after_scope() {
        let file_locks: FileLocks = Arc::new(Mutex::new(HashMap::new()));
        let path = PathBuf::from("/tmp/release_test.dart");

        // Acquire and release lock in inner scope
        {
            let file_lock = get_file_lock(&file_locks, &path).await;
            let _guard = file_lock.lock().await;
            // Lock is held here
        }
        // Lock should be released

        // Should be able to acquire immediately
        let file_lock = get_file_lock(&file_locks, &path).await;
        let guard = file_lock.try_lock();

        assert!(
            guard.is_ok(),
            "Lock should be available after previous guard dropped"
        );
    }
}
