# Development

## Toolchain

- Node.js 24 or newer
- Rust stable (`rustup` recommended)
- Platform-specific [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/)

On Ubuntu/Debian:

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

## Commands

```bash
npm ci                 # exact frontend dependencies
npm run dev            # browser UI with deterministic mock PTY
npm run tauri dev      # real native app and shell
npm run verify         # types, frontend tests, production UI build
npm run tauri build    # current-platform packages
```

Native checks:

```bash
cd src-tauri
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

## Browser mock

Opening the Vite URL outside Tauri starts a local deterministic PowerShell mock. It exists only for fast visual QA; packaged builds detect Tauri and always use the Rust PTY commands.

## Live PTY integration test

`src/lib/thread-mode.e2e.test.ts` runs the real thread-mode data path — decoding, terminal-query replies, and OSC 777 marker parsing — against a real PowerShell spawned under a real ConPTY. It is skipped by default so `npm test` stays hermetic. To run it you need a small probe binary that bridges a ConPTY to stdio using the same `portable-pty` crate as the app:

```bash
mkdir -p /tmp/ptyprobe/src && cd /tmp/ptyprobe
cat > Cargo.toml <<'EOF'
[package]
name = "ptyprobe"
version = "0.1.0"
edition = "2021"

[dependencies]
portable-pty = "0.9"
EOF
# src/main.rs: spawn powershell.exe -NoLogo -NoExit [-File $script], then
# with RAW=1 pipe PTY output to stdout and stdin to the PTY, and exit when
# stdin closes.
cargo build --release
CHATTERM_PTY_PROBE=/tmp/ptyprobe/target/release/ptyprobe.exe \
CHATTERM_SCRIPT="$APPDATA/xyz.temidayo.chatterm/shell-integration/chatterm.ps1" \
npx vitest run src/lib/thread-mode.e2e.test.ts
```

Windows only; macOS and Linux shells do not block on the query this guards against.

## Release process

1. Update versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Move changelog entries out of Unreleased and update `RELEASE_NOTES.md`.
3. Run all checks and build a local native package.
4. Commit and push, then create an annotated tag such as `v0.1.0`.
5. Push the tag. `release.yml` builds Windows, both macOS architectures, and Linux, then attaches packages to the GitHub Release.

Code-signing secrets can be added later using Tauri’s documented environment variables; never commit certificates or passwords.
