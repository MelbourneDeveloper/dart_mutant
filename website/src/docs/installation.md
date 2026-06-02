---
layout: docs.njk
title: Installation
description: Install dart_mutant on macOS and Linux via Homebrew, on Windows via Scoop, or build from source on any platform.
---

# Installation

## macOS (Apple Silicon) and Linux (x64) — Homebrew

Install via the [Nimblesite Homebrew tap](https://github.com/Nimblesite/homebrew-tap):

```bash
brew install nimblesite/tap/dart_mutant
```

This works on macOS (Apple Silicon) and any Linux distro with
[Homebrew on Linux](https://docs.brew.sh/Homebrew-on-Linux) installed.

To upgrade:

```bash
brew update
brew upgrade dart_mutant
```

## Windows (x64) — Scoop

Install via the [Nimblesite Scoop bucket](https://github.com/Nimblesite/scoop-bucket):

```powershell
scoop bucket add nimblesite https://github.com/Nimblesite/scoop-bucket
scoop install dart_mutant
```

To upgrade:

```powershell
scoop update dart_mutant
```

## Pre-built Binaries

Pre-built binaries are published on the
[releases page](https://github.com/Nimblesite/dart_mutant/releases) for:

- `aarch64-apple-darwin` (macOS, Apple Silicon)
- `x86_64-unknown-linux-gnu` (Linux, x86_64)
- `x86_64-pc-windows-msvc` (Windows, x86_64)

Intel macOS and ARM Linux users should install [from source](#build-from-source).

```bash
# macOS (Apple Silicon)
VERSION=v0.4.0
curl -L "https://github.com/Nimblesite/dart_mutant/releases/download/${VERSION}/dart_mutant-${VERSION}-aarch64-apple-darwin.tar.gz" | tar xz
sudo mv dart_mutant /usr/local/bin/

# Linux (x86_64)
VERSION=v0.4.0
curl -L "https://github.com/Nimblesite/dart_mutant/releases/download/${VERSION}/dart_mutant-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar xz
sudo mv dart_mutant /usr/local/bin/
```

## Build from Source

Requires [Rust](https://rustup.rs/) 1.75+:

```bash
git clone https://github.com/Nimblesite/dart_mutant
cd dart_mutant
cargo build --release

# Add to PATH
export PATH="$PATH:$(pwd)/target/release"
```

## Verify Installation

```bash
dart_mutant --version
# dart_mutant 0.4.0

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
