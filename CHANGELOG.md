# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-07

### Added
- **Core Architecture**: Tauri 2.x backend in Rust + Svelte 5 frontend with TypeScript & runes.
- **Safety Module**:
  - Hard-deny list protecting system directories (`/`, `/System`, `C:\Windows`, `.git/`, user home & Documents).
  - Symlink detection and traversal protection.
  - Multi-filesystem and cross-volume boundary protection.
  - Mandatory double-validation of target paths prior to deletion.
  - Safe simulation mode (Dry-run active by default during development).
- **Scanner Engine**:
  - Parallel background scanner utilizing `jwalk` and `rayon` with throttle event emissions (`scan://progress`).
  - Global cache scanning for package managers (npm, pnpm, Yarn, Bun, Cargo), browsers (Playwright, Puppeteer), IDEs (VS Code, Xcode), and Docker.
  - Project marker detection (`package.json`, `Cargo.toml`, `composer.json`, `pyproject.toml`, `go.mod`).
  - Stale project detection based on last activity and configurable day threshold.
  - Accurate physical disk size calculation with inode deduplication for pnpm hardlinks.
- **Cleaner Engine**:
  - Item ID-only API preventing arbitrary frontend path injection.
  - Multi-method cleaning: `trash` via trash-rs crate, permanent `delete` for caches, and CLI `command` execution with timeout.
  - Real-time progress modal with cancel support.
  - Results inspector with terminal output display for CLI commands.
- **History & Settings**:
  - Persistent JSON history tracking freed disk space, date/time, and item details.
  - Settings manager with project roots picker, exclude paths manager, scan depth limit, and dry-run toggle.
- **macOS Integration**:
  - Full Disk Access detection (`~/Library/Safari`) and direct navigation to macOS Privacy & Security Settings.
- **Cross-Platform Support**:
  - Category definitions for macOS, Linux, and Windows.
  - Universal path and environment variable expansion (`%LOCALAPPDATA%`, `%APPDATA%`, `%USERPROFILE%`, `%TEMP%`).
  - Multi-platform GitHub Actions CI matrix (`macos-latest`, `ubuntu-22.04`, `windows-latest`).
