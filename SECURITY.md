# Security policy

## Supported versions

Security fixes are provided for the latest released version of Chatterm.

## Reporting a vulnerability

Please do not open a public issue for an unpatched vulnerability. Use GitHub’s **Security → Report a vulnerability** private advisory flow for this repository. Include affected versions, operating system, reproduction conditions, impact, and a proposed mitigation if known.

You should receive an acknowledgement within seven days. We will investigate, coordinate a fix and release, then credit reporters who want attribution.

## Trust boundary

Chatterm is a terminal emulator: commands intentionally execute with the privileges of the user who launched it. A command doing what it was asked to do is not a sandbox escape. Relevant reports include unintended remote communication, terminal-output injection that crosses into privileged application behavior, unsafe history disclosure, shell-integration command execution not initiated by the user, or Tauri capability bypasses.

Never include live credentials, private keys, or sensitive terminal history in a report.
