# Chatterm v0.1.0 — First public release

Chatterm makes the command line easier to read without replacing it. One native shell can move between structured command blocks, a conversational layout, and a full terminal surface.

## Highlights

- **One live PTY:** Thread, Chat, and Focus share the same process and state.
- **Readable history:** commands keep their output, exit code, and duration together.
- **Real terminal behavior:** Focus supports terminal applications, ANSI color, resize, search, selection, and keyboard input.
- **Local by design:** no account, telemetry, cloud, analytics, or automatic history capture.
- **Cross-platform:** automated packages for Windows x64, macOS Apple silicon/Intel, and Linux x64.

## Platform notes

These first community packages are not code-signed or notarized.

- On Windows, SmartScreen may show “Windows protected your PC.” Choose **More info → Run anyway** only if the file came from this repository.
- On macOS, right-click Chatterm and choose **Open** the first time if Gatekeeper blocks the unsigned app.
- On Linux, make the AppImage executable with `chmod +x Chatterm_*.AppImage` before launching it.

## Shell support

Structured Thread metadata is enabled for PowerShell/Windows PowerShell, Bash, Zsh, and Fish. Other shells remain usable in Focus mode. Shell startup files are preserved and sourced by Chatterm’s small integration wrapper.

## Known boundaries

- Packages are unsigned until community signing infrastructure is funded/configured.
- Saved sessions are local to one machine; sync/export is not included in v0.1.
- Very long live sessions retain a bounded recent raw buffer (12 MiB) and 10,000 lines of Focus scrollback.

Please report reproducible problems through [GitHub Issues](https://github.com/temidayoxyz/chatterm/issues) and use the private process in [SECURITY.md](SECURITY.md) for vulnerabilities.
