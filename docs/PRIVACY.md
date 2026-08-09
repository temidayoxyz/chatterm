# Privacy design

Chatterm does not have a privacy toggle because private, local behavior is the default architecture.

## Data processed

- Terminal input and output for currently open sessions
- Shell name, current working directory, timing, and exit status
- UI settings stored in WebView local storage
- Sessions the user explicitly saves to a local SQLite database

## Data not collected

Chatterm includes no account, telemetry, analytics SDK, crash-upload service, cloud synchronization, advertising identifier, remote font, or AI provider. The application CSP does not allow general outbound connections.

## Retention and deletion

Unsaved session data exists in application memory and disappears when the session/app closes. The recent raw buffer is bounded at 12 MiB per live session. Saved sessions remain in the local database until the user deletes them from the sidebar.

## Shell behavior

Your shell startup files and commands can independently access the network or filesystem, exactly as they can in another terminal. Chatterm does not intercept or proxy that activity. Installing extensions or modifying shell profiles is outside Chatterm’s automatic behavior.
