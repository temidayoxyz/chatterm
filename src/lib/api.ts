import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  CreateSessionRequest,
  SaveSessionRequest,
  SavedSessionDetail,
  SavedSessionSummary,
  SessionEvent,
  SessionInfo,
  ShellProfile,
} from "./types";

const mockListeners = new Map<string, (event: SessionEvent) => void>();

export function isDesktopRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function listShellProfiles(): Promise<ShellProfile[]> {
  if (!isDesktopRuntime()) {
    return [
      {
        id: "pwsh",
        name: "PowerShell",
        program: "pwsh",
        enhancedThreads: true,
      },
    ];
  }
  return invoke<ShellProfile[]>("list_shell_profiles");
}

export async function createSession(
  request: CreateSessionRequest,
  onEvent: (event: SessionEvent) => void,
): Promise<SessionInfo> {
  if (!isDesktopRuntime()) return createMockSession(request, onEvent);
  const channel = new Channel<SessionEvent>();
  channel.onmessage = onEvent;
  return invoke<SessionInfo>("create_session", { request, onEvent: channel });
}

export async function writeToSession(sessionId: string, data: string): Promise<void> {
  if (!isDesktopRuntime()) {
    mockWrite(sessionId, data);
    return;
  }
  await invoke("write_to_session", { sessionId, data });
}

export async function resizeSession(
  sessionId: string,
  rows: number,
  cols: number,
): Promise<void> {
  if (!isDesktopRuntime()) return;
  await invoke("resize_session", { sessionId, rows, cols });
}

export async function closeSession(sessionId: string): Promise<void> {
  mockListeners.delete(sessionId);
  if (!isDesktopRuntime()) return;
  await invoke("close_session", { sessionId });
}

export async function saveSession(
  request: SaveSessionRequest,
): Promise<SavedSessionSummary> {
  if (!isDesktopRuntime()) {
    const summary = {
      id: request.id,
      name: request.name,
      shellName: request.shellName,
      cwd: request.cwd,
      commandCount: request.blocks.length,
      savedAt: new Date().toISOString(),
    };
    localStorage.setItem(`chatterm:saved:${request.id}`, JSON.stringify({ ...request, ...summary }));
    return summary;
  }
  return invoke<SavedSessionSummary>("save_session", { request });
}

export async function listSavedSessions(): Promise<SavedSessionSummary[]> {
  if (!isDesktopRuntime()) {
    return Object.keys(localStorage)
      .filter((key) => key.startsWith("chatterm:saved:"))
      .map((key) => JSON.parse(localStorage.getItem(key) ?? "{}") as SavedSessionSummary)
      .sort((a, b) => b.savedAt.localeCompare(a.savedAt));
  }
  return invoke<SavedSessionSummary[]>("list_saved_sessions");
}

export async function loadSavedSession(id: string): Promise<SavedSessionDetail> {
  if (!isDesktopRuntime()) {
    const value = localStorage.getItem(`chatterm:saved:${id}`);
    if (!value) throw new Error("Saved session not found");
    return JSON.parse(value) as SavedSessionDetail;
  }
  return invoke<SavedSessionDetail>("load_saved_session", { id });
}

export async function deleteSavedSession(id: string): Promise<void> {
  if (!isDesktopRuntime()) {
    localStorage.removeItem(`chatterm:saved:${id}`);
    return;
  }
  await invoke("delete_saved_session", { id });
}

function createMockSession(
  request: CreateSessionRequest,
  onEvent: (event: SessionEvent) => void,
): SessionInfo {
  const id = crypto.randomUUID();
  mockListeners.set(id, onEvent);
  const session = {
    id,
    shellId: request.shellId ?? "pwsh",
    shellName: "PowerShell",
    enhancedThreads: true,
    cwd: request.cwd ?? "~/projects/chatterm",
    createdAt: new Date().toISOString(),
  };
  window.setTimeout(() => {
    emitMock(id, `\u001b]777;A\u0007\u001b[38;2;175;167;255mPS\u001b[0m ${session.cwd}> `);
  }, 120);
  return session;
}

function mockWrite(sessionId: string, data: string): void {
  const command = data.replace(/[\r\n]+$/, "").trim();
  if (!command) return;
  emitMock(sessionId, `\r\n\u001b]777;C;${command}\u0007`);
  window.setTimeout(() => emitMock(sessionId, mockOutput(command)), 140);
  window.setTimeout(() => {
    emitMock(
      sessionId,
      `\u001b]777;D;0\u0007\u001b]777;A\u0007\r\n\u001b[38;2;175;167;255mPS\u001b[0m ~/projects/chatterm> `,
    );
  }, 520);
}

function mockOutput(command: string): string {
  if (command.includes("npm install")) {
    return "\r\nadded 324 packages\r\naudited 324 packages\r\n\u001b[32mfound 0 vulnerabilities\u001b[0m\r\n";
  }
  if (command.includes("npm run build")) {
    return "\r\nBuilding production bundle...\r\n\u001b[32m✓\u001b[0m /\r\n\u001b[32m✓\u001b[0m /dashboard\r\n\u001b[32m✓\u001b[0m /settings\r\n";
  }
  if (command === "clear" || command === "cls") return "\u001b[2J\u001b[H";
  return `\r\n\u001b[38;2;120;205;181m✓\u001b[0m ${command}\r\nCommand completed in the browser preview.\r\n`;
}

function emitMock(sessionId: string, text: string): void {
  const listener = mockListeners.get(sessionId);
  if (!listener) return;
  listener({
    event: "data",
    sessionId,
    data: Array.from(new TextEncoder().encode(text)),
  });
}
