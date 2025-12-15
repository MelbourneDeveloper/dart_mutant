---
layout: docs.njk
title: Installation
---

# Installation

## Homebrew (macOS)

The easiest way to install on macOS:

```bash
brew tap MelbourneDeveloper/tap
brew install dart_mutant
```

To upgrade:

```bash
brew upgrade dart_mutant
```

## Pre-built Binaries

Download the latest release from the [releases page](https://github.com/MelbourneDeveloper/dart_mutant/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/MelbourneDeveloper/dart_mutant/releases/latest/download/dart_mutant-aarch64-apple-darwin.tar.gz | tar xz
sudo mv dart_mutant /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/MelbourneDeveloper/dart_mutant/releases/latest/download/dart_mutant-x86_64-apple-darwin.tar.gz | tar xz
sudo mv dart_mutant /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/MelbourneDeveloper/dart_mutant/releases/latest/download/dart_mutant-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv dart_mutant /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/MelbourneDeveloper/dart_mutant/releases/latest/download/dart_mutant-x86_64-pc-windows-msvc.zip -OutFile dart_mutant.zip
Expand-Archive dart_mutant.zip
Move-Item dart_mutant\dart_mutant.exe C:\Windows\
```

**Note:** For direct binary downloads, check the [releases page](https://github.com/MelbourneDeveloper/dart_mutant/releases) for the exact filename which includes the version number.

## Build from Source

Requires [Rust](https://rustup.rs/) 1.75+:

```bash
git clone https://github.com/MelbourneDeveloper/dart_mutant
cd dart_mutant
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## Verify Installation

```bash
dart_mutant --version
# dart_mutant 0.1.0

dart_mutant --help
# Usage: dart_mutant [OPTIONS]
# ...
```

## Requirements

- **Dart SDK**: dart_mutant runs `dart test` internally, so you need Dart installed
- **Test suite**: Your project should have tests in `test/` directory
- **Git** (optional): Required for `--incremental` mode

## Next Steps

- [Quick Start](/docs/quickstart/) - Run your first mutation test
