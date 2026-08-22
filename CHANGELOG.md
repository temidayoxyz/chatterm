# Changelog

All notable changes to Chatterm are documented here. The project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.1] - 2026-08-22

### Fixed

- Commands in thread mode no longer hang with no response. Session events were serialized with snake_case field names (`session_id`), so the frontend silently ignored every one of them — blocks never captured output or exit status, and Windows PowerShell's PSReadLine, which blocks until a host answers its startup cursor-position query (`ESC[6n`), never received a reply in thread mode where no terminal emulator is attached. Events now serialize camelCase, and Chatterm answers terminal queries itself whenever a session has no live terminal surface.
- The sidebar and top bar no longer report a still-starting shell as "Ready"; it shows "Starting…" until the first prompt marker arrives.

### Added

- Live PowerShell integration test covering thread-mode decoding, terminal-query replies, and OSC 777 marker parsing (see docs/DEVELOPMENT.md).

## [0.1.0] - 2026-08-09

### Added

- Native Windows, macOS, and Linux terminal sessions powered by `portable-pty`
- Thread mode with Blocks and Chat layouts over the same live shell
- Focus mode with xterm.js and automatic alternate-screen switching
- PowerShell, Bash, Zsh, and Fish structured command markers
- Live working-directory tracking and command exit status/duration
- Explicit local SQLite session saving and read-only saved-session playback
- Terminal/output search, command re-run, copy, interrupt, and bounded scrollback
- Basalt, Paper, and Classic themes with offline typography
- Strict Content Security Policy and minimal Tauri capability grants
- Cross-platform CI and tag-driven GitHub Release packaging

[Unreleased]: https://github.com/temidayoxyz/chatterm/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/temidayoxyz/chatterm/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/temidayoxyz/chatterm/releases/tag/v0.1.0
