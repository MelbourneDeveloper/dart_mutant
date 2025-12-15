//! Embedded CSS for HTML reports
//!
//! This CSS matches the dart_mutant website's Toxic Lab theme.

/// Get the complete CSS for the HTML report
pub fn get_report_css() -> &'static str {
    r#"/* ============================================
   DART MUTANT - Mutation Testing Report
   Theme: Toxic Lab / Bio-hazard Sci-Fi
   ============================================ */

/* CSS Custom Properties */
:root {
  /* Primary - Toxic Green (mutation/bio-hazard) */
  --color-primary: #39ff14;
  --color-primary-dark: #2ed10f;
  --color-primary-glow: rgba(57, 255, 20, 0.3);

  /* Secondary - Warning Orange (danger/alert) */
  --color-secondary: #ff6b35;
  --color-secondary-dark: #e55a2b;
  --color-secondary-glow: rgba(255, 107, 53, 0.3);

  /* Accent - Cyber Cyan */
  --color-accent: #00fff5;
  --color-accent-dark: #00d4cc;

  /* Status Colors */
  --color-killed: #39ff14;
  --color-survived: #ff6b35;
  --color-timeout: #ffd93d;
  --color-error: #ff3131;
  --color-no-coverage: #6b7280;

  /* Backgrounds - Dark Lab */
  --bg-darkest: #0a0a0f;
  --bg-dark: #0f1117;
  --bg-medium: #161922;
  --bg-light: #1e222d;
  --bg-card: #1a1d26;

  /* Text */
  --text-primary: #e8e8e8;
  --text-secondary: #9ca3af;
  --text-muted: #6b7280;

  /* Borders */
  --border-color: #2a2f3a;
  --border-glow: var(--color-primary);

  /* Fonts */
  --font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', 'SF Mono', Consolas, monospace;
  --font-display: 'Orbitron', var(--font-sans);

  /* Spacing */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
  --spacing-2xl: 3rem;

  /* Border Radius */
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;
  --radius-xl: 16px;

  /* Transitions */
  --transition-fast: 150ms ease;
  --transition-normal: 250ms ease;
}

/* Reset & Base */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html {
  scroll-behavior: smooth;
  font-size: 16px;
}

body {
  font-family: var(--font-sans);
  background-color: var(--bg-darkest);
  color: var(--text-primary);
  line-height: 1.6;
  min-height: 100vh;
  overflow-x: hidden;
}

/* Background Grid Pattern */
body::before {
  content: '';
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image:
    linear-gradient(rgba(57, 255, 20, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(57, 255, 20, 0.03) 1px, transparent 1px);
  background-size: 50px 50px;
  pointer-events: none;
  z-index: -1;
}

/* Typography */
h1, h2, h3 {
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: 0.02em;
}

a {
  color: var(--color-primary);
  text-decoration: none;
  transition: color var(--transition-fast);
}

a:hover {
  color: var(--color-accent);
}

code, pre {
  font-family: var(--font-mono);
}

code {
  background: var(--bg-medium);
  padding: 0.2em 0.4em;
  border-radius: var(--radius-sm);
  font-size: 0.9em;
  color: var(--color-primary);
}

/* Layout */
.container {
  width: 100%;
  max-width: 1400px;
  margin: 0 auto;
  padding: var(--spacing-xl);
}

/* Header */
.header {
  text-align: center;
  margin-bottom: var(--spacing-2xl);
  padding: var(--spacing-2xl);
  background: var(--bg-dark);
  border-radius: var(--radius-xl);
  border: 1px solid var(--border-color);
  position: relative;
  overflow: hidden;
}

.header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, var(--color-primary), var(--color-secondary), var(--color-primary));
}

.header::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, var(--color-primary-glow) 0%, transparent 70%);
  pointer-events: none;
  opacity: 0.3;
}

.logo {
  position: relative;
  z-index: 1;
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--color-primary);
  text-shadow: 0 0 30px var(--color-primary-glow);
  margin-bottom: var(--spacing-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-sm);
}

.logo-icon {
  font-size: 2.5rem;
}

.tagline {
  position: relative;
  z-index: 1;
  color: var(--text-secondary);
  font-size: 1.1rem;
}

/* Score Card */
.score-card {
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: var(--spacing-xl);
  margin-bottom: var(--spacing-2xl);
}

@media (max-width: 900px) {
  .score-card {
    grid-template-columns: 1fr;
  }
}

.score-display {
  background: var(--bg-card);
  border-radius: var(--radius-xl);
  padding: var(--spacing-2xl);
  text-align: center;
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.score-display::before {
  content: '';
  position: absolute;
  inset: 0;
  background: radial-gradient(circle at center, var(--color-primary-glow) 0%, transparent 70%);
  opacity: 0.1;
}

.score-label {
  position: relative;
  font-size: 0.875rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.15em;
  margin-bottom: var(--spacing-sm);
}

.score-value {
  position: relative;
  font-size: 5rem;
  font-weight: 700;
  line-height: 1;
}

.score-value.high { color: var(--color-killed); text-shadow: 0 0 40px var(--color-primary-glow); }
.score-value.medium { color: var(--color-timeout); text-shadow: 0 0 40px rgba(255, 217, 61, 0.3); }
.score-value.low { color: var(--color-survived); text-shadow: 0 0 40px var(--color-secondary-glow); }

.score-bar {
  position: relative;
  width: 100%;
  height: 8px;
  background: var(--bg-medium);
  border-radius: 100px;
  overflow: hidden;
  margin-top: var(--spacing-lg);
}

.score-bar-fill {
  height: 100%;
  border-radius: 100px;
  transition: width 1s ease-out;
}

.score-bar-fill.high { background: linear-gradient(90deg, var(--color-killed), var(--color-accent)); }
.score-bar-fill.medium { background: linear-gradient(90deg, var(--color-timeout), #ffaa00); }
.score-bar-fill.low { background: linear-gradient(90deg, var(--color-survived), #ff9500); }

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: var(--spacing-md);
}

.stat-card {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  text-align: center;
  border: 1px solid var(--border-color);
  transition: all var(--transition-normal);
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
}

.stat-value {
  font-size: 2.5rem;
  font-weight: 700;
  margin-bottom: var(--spacing-xs);
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.stat-total .stat-value { color: var(--text-primary); }
.stat-killed .stat-value { color: var(--color-killed); }
.stat-killed:hover { border-color: var(--color-killed); }
.stat-survived .stat-value { color: var(--color-survived); }
.stat-survived:hover { border-color: var(--color-survived); }
.stat-timeout .stat-value { color: var(--color-timeout); }
.stat-timeout:hover { border-color: var(--color-timeout); }
.stat-no-coverage .stat-value { color: var(--color-no-coverage); }
.stat-error .stat-value { color: var(--color-error); }

/* Section Title */
.section-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: var(--spacing-lg);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--text-primary);
}

.section-title::before {
  content: '';
  display: block;
  width: 4px;
  height: 28px;
  background: linear-gradient(180deg, var(--color-primary), var(--color-secondary));
  border-radius: 2px;
}

/* Filter Controls */
.filter-controls {
  display: flex;
  align-items: center;
  gap: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
  padding: var(--spacing-md) var(--spacing-lg);
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-color);
}

.filter-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.filter-checkbox {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  cursor: pointer;
  user-select: none;
}

.filter-checkbox input {
  width: 1.25rem;
  height: 1.25rem;
  accent-color: var(--color-primary);
  cursor: pointer;
}

.filter-checkbox span {
  font-size: 0.875rem;
  color: var(--text-primary);
}

/* File Cards */
.file-card {
  background: var(--bg-card);
  border-radius: var(--radius-lg);
  margin-bottom: var(--spacing-md);
  border: 1px solid var(--border-color);
  overflow: hidden;
  transition: border-color var(--transition-fast);
}

.file-card:hover {
  border-color: var(--color-primary);
}

.file-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.file-header:hover {
  background: var(--bg-light);
}

.file-name {
  font-family: var(--font-mono);
  font-size: 0.9rem;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.file-name::before {
  content: '📄';
}

.file-stats {
  display: flex;
  align-items: center;
  gap: var(--spacing-lg);
}

.file-mutants {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.file-score {
  font-weight: 600;
  padding: var(--spacing-xs) var(--spacing-md);
  border-radius: 100px;
  font-size: 0.875rem;
}

.file-score.high {
  background: rgba(57, 255, 20, 0.15);
  color: var(--color-killed);
  border: 1px solid var(--color-killed);
}

.file-score.medium {
  background: rgba(255, 217, 61, 0.15);
  color: var(--color-timeout);
  border: 1px solid var(--color-timeout);
}

.file-score.low {
  background: rgba(255, 107, 53, 0.15);
  color: var(--color-survived);
  border: 1px solid var(--color-survived);
}

.file-content {
  display: none;
  padding: var(--spacing-md) var(--spacing-lg);
  border-top: 1px solid var(--border-color);
  background: var(--bg-darkest);
}

.file-card.expanded .file-content {
  display: block;
}

.file-card.expanded .file-header {
  background: var(--bg-light);
}

/* Mutant Items */
.mutant-item {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  border-radius: var(--radius-md);
  margin-bottom: var(--spacing-sm);
  background: var(--bg-card);
  border-left: 3px solid transparent;
  transition: all var(--transition-fast);
}

.mutant-item:hover {
  background: var(--bg-light);
}

.mutant-item.killed {
  border-left-color: var(--color-killed);
}

.mutant-item.survived {
  border-left-color: var(--color-survived);
}

.mutant-item.timeout {
  border-left-color: var(--color-timeout);
}

.mutant-item.error, .mutant-item.no-coverage {
  border-left-color: var(--color-error);
}

.mutant-status {
  font-size: 1.5rem;
  width: 2.5rem;
  text-align: center;
  flex-shrink: 0;
}

.mutant-details {
  flex: 1;
  min-width: 0;
}

.mutant-location {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-bottom: var(--spacing-xs);
}

.mutant-description {
  font-size: 0.9rem;
  color: var(--text-primary);
  margin-bottom: var(--spacing-sm);
}

.mutant-code {
  font-family: var(--font-mono);
  font-size: 0.85rem;
  padding: var(--spacing-md);
  background: var(--bg-darkest);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  overflow-x: auto;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.code-original {
  color: var(--color-survived);
  text-decoration: line-through;
  opacity: 0.8;
}

.code-arrow {
  color: var(--text-muted);
}

.code-replacement {
  color: var(--color-killed);
}

/* Hidden states */
.mutant-item.hidden {
  display: none;
}

.file-card.all-hidden {
  display: none;
}

/* Footer */
.footer {
  margin-top: var(--spacing-2xl);
  padding: var(--spacing-lg);
  background: var(--bg-dark);
  border-radius: var(--radius-lg);
  text-align: center;
  color: var(--text-secondary);
  font-size: 0.875rem;
  border: 1px solid var(--border-color);
}

.footer a {
  color: var(--color-primary);
  font-weight: 500;
}

.footer a:hover {
  color: var(--color-accent);
  text-decoration: underline;
}

/* Responsive */
@media (max-width: 640px) {
  .container {
    padding: var(--spacing-md);
  }

  .header {
    padding: var(--spacing-lg);
  }

  .logo {
    font-size: 1.75rem;
  }

  .score-value {
    font-size: 3.5rem;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .filter-controls {
    flex-direction: column;
    align-items: flex-start;
  }

  .file-header {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--spacing-sm);
  }

  .file-stats {
    width: 100%;
    justify-content: space-between;
  }
}

/* Animation */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.file-card {
  animation: fadeIn 0.3s ease forwards;
}

.file-card:nth-child(1) { animation-delay: 0s; }
.file-card:nth-child(2) { animation-delay: 0.05s; }
.file-card:nth-child(3) { animation-delay: 0.1s; }
.file-card:nth-child(4) { animation-delay: 0.15s; }
.file-card:nth-child(5) { animation-delay: 0.2s; }
"#
}
