import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { spawn, type ChildProcess } from "node:child_process";
import { existsSync } from "node:fs";
import { ChattermProtocolParser, type ProtocolEvent } from "./protocol";
import { TerminalQueryResponder } from "./terminal-host";

/**
 * End-to-end thread-mode simulation against a real Windows PowerShell under a
 * real ConPTY. The probe binary (see docs/DEVELOPMENT.md) bridges PTY stdio;
 * this test runs the exact frontend data path the app uses: decode chunks,
 * answer terminal queries, parse OSC 777 markers into blocks. Skipped unless
 * CHATTERM_PTY_PROBE points at the probe and CHATTERM_SCRIPT at an installed
 * integration script, so plain `npm test` stays hermetic.
 */
const probe = process.env.CHATTERM_PTY_PROBE;
const script = process.env.CHATTERM_SCRIPT;
const enabled = Boolean(probe && script && existsSync(probe));

describe.skipIf(!enabled)("thread mode against a live PowerShell", () => {
  let shell: ChildProcess;
  let responder: TerminalQueryResponder;
  let parser: ChattermProtocolParser;
  let decoder: TextDecoder;
  const events: ProtocolEvent[] = [];
  const output: string[] = [];

  beforeAll(() => {
    shell = spawn(probe!, ["file"], {
      env: { ...process.env, RAW: "1", CHATTERM_SCRIPT: script! },
      stdio: ["pipe", "pipe", "pipe"],
    });
    responder = new TerminalQueryResponder();
    parser = new ChattermProtocolParser();
    decoder = new TextDecoder();
    shell.stdout!.on("data", (chunk: Buffer) => {
      const raw = decoder.decode(chunk, { stream: true });
      for (const reply of responder.push(raw)) shell.stdin!.write(reply);
      const result = parser.push(raw);
      for (const part of result.parts) {
        if (part.type === "text") output.push(part.value);
        else events.push(part.value);
      }
    });
  });

  afterAll(() => {
    shell.stdin?.end();
    shell.kill();
  });

  async function waitFor(check: () => boolean, timeoutMs: number, what: string) {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (check()) return;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    throw new Error(`timed out waiting for ${what}`);
  }

  it(
    "boots to a prompt, runs a command, and reports its exit",
    { timeout: 45_000 },
    async () => {
      // Without query replies Windows PowerShell's PSReadLine never shows a
      // prompt; with them the promptStart/promptEnd markers must arrive.
      await waitFor(
        () => events.some((event) => event.type === "promptEnd"),
        15_000,
        "the first prompt (shell answered its cursor-position query)",
      );

      shell.stdin!.write("echo chatterm-e2e-ok\r");

      await waitFor(
        () => events.some((event) => event.type === "commandEnd"),
        15_000,
        "the command to finish",
      );

      const command = events.find((event) => event.type === "commandStart");
      expect(command).toMatchObject({ type: "commandStart", command: "echo chatterm-e2e-ok" });
      // After commandEnd the shell redraws its prompt, so a trailing
      // promptStart/promptEnd pair is expected to follow it.
      const end = events.find((event) => event.type === "commandEnd");
      expect(end).toMatchObject({ type: "commandEnd", exitCode: 0 });
      expect(output.join("")).toContain("chatterm-e2e-ok");
    },
  );
});
