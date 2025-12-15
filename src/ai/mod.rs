//! AI-powered smart mutation placement
//!
//! Uses LLMs to analyze code and suggest high-value mutation locations.
//! This helps find mutations that are more likely to catch weak tests.

use crate::cli::AiProvider;
use crate::mutation::{Mutation, MutationOperator, SourceLocation};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// A single mutation suggestion from AI
#[derive(Debug, Clone, Deserialize)]
pub struct MutationSuggestion {
    pub line: usize,
    pub column: usize,
    pub original: String,
    pub mutated: String,
    pub reason: String,
    pub confidence: f64,
}

/// AI-powered mutation suggester
pub struct AiMutationSuggester {
    provider: AiProvider,
    api_key: Option<String>,
    ollama_url: String,
    ollama_model: String,
    max_per_file: usize,
}

impl AiMutationSuggester {
    /// Create a new AI mutation suggester
    pub fn new(
        provider: AiProvider,
        api_key: Option<String>,
        ollama_url: String,
        ollama_model: String,
        max_per_file: usize,
    ) -> Self {
        Self {
            provider,
            api_key,
            ollama_url,
            ollama_model,
            max_per_file,
        }
    }

    /// Suggest high-value mutations for a Dart file
    pub async fn suggest_mutations(&self, file_path: &Path, source: &str) -> Result<Vec<Mutation>> {
        let suggestions = match self.provider {
            AiProvider::Anthropic => self.suggest_with_anthropic(source).await?,
            AiProvider::OpenAI => self.suggest_with_openai(source).await?,
            AiProvider::Ollama => self.suggest_with_ollama(source).await?,
            AiProvider::None => return Ok(vec![]),
        };

        // Convert suggestions to mutations
        let mutations: Vec<Mutation> = suggestions
            .into_iter()
            .take(self.max_per_file)
            .filter_map(|s| self.suggestion_to_mutation(file_path, source, s))
            .collect();

        Ok(mutations)
    }

    fn suggestion_to_mutation(
        &self,
        file_path: &Path,
        source: &str,
        suggestion: MutationSuggestion,
    ) -> Option<Mutation> {
        // Find the byte offset for the given line and column
        let lines: Vec<&str> = source.lines().collect();
        if suggestion.line == 0 || suggestion.line > lines.len() {
            return None;
        }

        let mut byte_start = 0;
        for (i, line) in lines.iter().enumerate() {
            if i + 1 == suggestion.line {
                // Find the column within this line
                let col_offset = suggestion.column.saturating_sub(1);
                if col_offset < line.len() {
                    byte_start += col_offset;

                    // Find where the original text ends
                    let remaining = &source[byte_start..];
                    if remaining.starts_with(&suggestion.original) {
                        let byte_end = byte_start + suggestion.original.len();

                        return Some(Mutation {
                            id: format!(
                                "ai-{:x}",
                                md5::compute(format!(
                                    "{}:{}:{}",
                                    file_path.display(),
                                    suggestion.line,
                                    suggestion.original
                                ))
                            ),
                            location: SourceLocation {
                                file: file_path.to_path_buf(),
                                start_line: suggestion.line,
                                start_col: suggestion.column,
                                end_line: suggestion.line,
                                end_col: suggestion.column + suggestion.original.len(),
                                byte_start,
                                byte_end,
                            },
                            operator: MutationOperator::AiSuggested,
                            original: suggestion.original,
                            mutated: suggestion.mutated.clone(),
                            description: format!("AI: {}", suggestion.reason),
                            replacements: vec![suggestion.mutated],
                            ai_suggested: true,
                            ai_confidence: Some(suggestion.confidence),
                        });
                    }
                }
                break;
            }
            byte_start += line.len() + 1; // +1 for newline
        }

        None
    }

    async fn suggest_with_anthropic(&self, source: &str) -> Result<Vec<MutationSuggestion>> {
        let api_key = self
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .context("Anthropic API key not set. Use --ai-key or ANTHROPIC_API_KEY env var")?;

        let prompt = self.build_prompt(source);

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await
            .context("Failed to call Anthropic API")?;

        let body: serde_json::Value = response.json().await?;
        self.parse_ai_response(&body)
    }

    async fn suggest_with_openai(&self, source: &str) -> Result<Vec<MutationSuggestion>> {
        let api_key = self
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .context("OpenAI API key not set. Use --ai-key or OPENAI_API_KEY env var")?;

        let prompt = self.build_prompt(source);

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "gpt-4-turbo-preview",
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "max_tokens": 4096,
                "temperature": 0.3
            }))
            .send()
            .await
            .context("Failed to call OpenAI API")?;

        let body: serde_json::Value = response.json().await?;
        self.parse_ai_response(&body)
    }

    async fn suggest_with_ollama(&self, source: &str) -> Result<Vec<MutationSuggestion>> {
        let prompt = self.build_prompt(source);

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/generate", self.ollama_url))
            .json(&serde_json::json!({
                "model": self.ollama_model,
                "prompt": prompt,
                "stream": false
            }))
            .send()
            .await
            .context("Failed to call Ollama API")?;

        let body: serde_json::Value = response.json().await?;
        self.parse_ai_response(&body)
    }

    fn build_prompt(&self, source: &str) -> String {
        format!(
            r#"Analyze this Dart code and suggest high-value mutation locations for mutation testing.

Focus on finding places where:
1. Boundary conditions are checked (off-by-one errors)
2. Boolean logic could be inverted
3. Arithmetic operations could be swapped
4. Null safety operators could be modified
5. String comparisons that matter for business logic
6. Collection operations that affect control flow

Return ONLY a JSON array of suggestions with this format:
```json
[
  {{
    "line": 10,
    "column": 5,
    "original": ">=",
    "mutated": ">",
    "reason": "Boundary check - off-by-one error",
    "confidence": 0.85
  }}
]
```

Rules:
- Only suggest mutations that would compile
- Focus on logic that tests should catch
- Confidence should reflect likelihood of catching weak tests
- Maximum {} suggestions
- Do NOT include any explanation text, ONLY the JSON array

Dart code:
```dart
{}
```"#,
            self.max_per_file, source
        )
    }

    fn parse_ai_response(&self, body: &serde_json::Value) -> Result<Vec<MutationSuggestion>> {
        // Try to extract content from different AI response formats
        let content = body
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|msg| msg.get("text"))
            .and_then(|t| t.as_str())
            // OpenAI format
            .or_else(|| {
                body.get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|choice| choice.get("message"))
                    .and_then(|msg| msg.get("content"))
                    .and_then(|t| t.as_str())
            })
            // Ollama format
            .or_else(|| body.get("response").and_then(|r| r.as_str()))
            .unwrap_or("");

        // Extract JSON from the response (handling markdown code blocks)
        let json_str = if let Some(start) = content.find('[') {
            if let Some(end) = content.rfind(']') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        // Parse the JSON
        let suggestions: Vec<MutationSuggestion> =
            serde_json::from_str(json_str).unwrap_or_default();

        Ok(suggestions)
    }
}

/// Convenience function to suggest mutations for multiple files
pub async fn suggest_mutations_for_files(
    files: &[PathBuf],
    provider: AiProvider,
    api_key: Option<String>,
    ollama_url: &str,
    ollama_model: &str,
    max_per_file: usize,
) -> Result<Vec<Mutation>> {
    if matches!(provider, AiProvider::None) {
        return Ok(vec![]);
    }

    let suggester = AiMutationSuggester::new(
        provider,
        api_key,
        ollama_url.to_string(),
        ollama_model.to_string(),
        max_per_file,
    );

    let mut all_mutations = Vec::new();

    for file in files {
        let source = std::fs::read_to_string(file)?;
        match suggester.suggest_mutations(file, &source).await {
            Ok(mutations) => all_mutations.extend(mutations),
            Err(e) => {
                tracing::warn!("Failed to get AI suggestions for {}: {}", file.display(), e);
            }
        }
    }

    Ok(all_mutations)
}
