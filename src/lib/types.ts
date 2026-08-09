export type AppMode = "thread" | "focus";
export type ThreadLayout = "blocks" | "chat";
export type BlockStatus = "running" | "success" | "failed" | "interrupted" | "unknown";

export interface ShellProfile {
  id: string;
  name: string;
  program: string;
  enhancedThreads: boolean;
}

export interface CreateSessionRequest {
  shellId?: string;
  cwd?: string;
  rows?: number;
  cols?: number;
}

export interface SessionInfo {
  id: string;
  shellId: string;
  shellName: string;
  enhancedThreads: boolean;
  cwd: string;
  createdAt: string;
}

export type SessionEvent =
  | { event: "data"; sessionId: string; data: number[] }
  | { event: "exit"; sessionId: string; code: number }
  | { event: "error"; sessionId: string; message: string };

export interface CommandBlock {
  id: string;
  command: string;
  output: string;
  status: BlockStatus;
  exitCode?: number;
  startedAt: string;
  endedAt?: string;
}

export interface UiSession extends SessionInfo {
  name: string;
  blocks: CommandBlock[];
  rawChunks: Uint8Array[];
  rawBytes: number;
  status: "starting" | "ready" | "running" | "exited" | "saved" | "error";
  currentBlockId?: string;
  saved: boolean;
  readOnly: boolean;
  error?: string;
}

export interface SaveSessionRequest {
  id: string;
  name: string;
  shellName: string;
  cwd: string;
  createdAt: string;
  blocks: CommandBlock[];
}

export interface SavedSessionSummary {
  id: string;
  name: string;
  shellName: string;
  cwd: string;
  commandCount: number;
  savedAt: string;
}

export interface SavedSessionDetail {
  id: string;
  name: string;
  shellName: string;
  cwd: string;
  createdAt: string;
  savedAt: string;
  blocks: CommandBlock[];
}

export interface AppSettings {
  theme: "basalt" | "paper" | "classic";
  fontSize: number;
  threadLayout: ThreadLayout;
  autoFocusInteractive: boolean;
  reducedMotion: boolean;
}
