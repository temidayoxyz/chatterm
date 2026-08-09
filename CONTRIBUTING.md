# Contributing to Chatterm

Thank you for helping make the command line calmer and more legible.

## Before opening a pull request

1. Search existing issues and discussions.
2. Keep changes focused; explain user-visible behavior and platform impact.
3. Add regression tests for protocol, persistence, or native logic changes.
4. Run the complete local verification:

```bash
npm ci
npm run verify
cd src-tauri
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

For UI changes, attach before/after screenshots at 1440×920 and 920×640. Check Basalt and Paper themes and keyboard focus states.

## Design principles

- Preserve real terminal semantics. Thread is a view over a PTY, not a command runner substitute.
- Keep command data local unless a future integration is explicit, opt-in, and isolated.
- Prefer quiet hierarchy, precise spacing, and the execution rail over generic cards or dashboard chrome.
- Never render terminal output as HTML.
- Treat Windows, macOS, and Linux as first-class; note anything you could not test.

## Commit and PR style

Use concise imperative commit subjects. A pull request should include the problem, the approach, verification evidence, and any security/privacy consequences.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).
