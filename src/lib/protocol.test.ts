import { describe, expect, it } from "vitest";
import { ChattermProtocolParser, stripTerminalControl } from "./protocol";

describe("ChattermProtocolParser", () => {
  it("removes protocol markers while preserving ordered text and events", () => {
    const parser = new ChattermProtocolParser();
    const result = parser.push("before\u001b]777;C;git status\u0007output\u001b]777;D;0\u0007after");

    expect(result.visible).toBe("beforeoutputafter");
    expect(result.events).toEqual([
      { type: "commandStart", command: "git status" },
      { type: "commandEnd", exitCode: 0 },
    ]);
    expect(result.parts).toEqual([
      { type: "text", value: "before" },
      { type: "event", value: { type: "commandStart", command: "git status" } },
      { type: "text", value: "output" },
      { type: "event", value: { type: "commandEnd", exitCode: 0 } },
      { type: "text", value: "after" },
    ]);
  });

  it("handles markers split across arbitrary chunks", () => {
    const parser = new ChattermProtocolParser();
    expect(parser.push("hello\u001b]77").visible).toBe("hello");
    const middle = parser.push("7;C;npm test\u001b");
    expect(middle.visible).toBe("");
    const end = parser.push("\\done");
    expect(end.events).toEqual([{ type: "commandStart", command: "npm test" }]);
    expect(end.visible).toBe("done");
  });

  it("ignores unknown private markers", () => {
    const parser = new ChattermProtocolParser();
    const result = parser.push("a\u001b]777;NOPE;secret\u0007b");
    expect(result.visible).toBe("ab");
    expect(result.events).toEqual([]);
  });

  it("reads a working directory without losing semicolons", () => {
    const parser = new ChattermProtocolParser();
    const result = parser.push("\u001b]777;A;C:\\work;bench\u0007");
    expect(result.events).toEqual([{ type: "promptStart", cwd: "C:\\work;bench" }]);
  });
});

describe("stripTerminalControl", () => {
  it("removes ANSI, OSC and unsafe control bytes without losing line structure", () => {
    const raw = "\u001b[31mred\u001b[0m\r\nnext\u001b]0;title\u0007\u0001";
    expect(stripTerminalControl(raw)).toBe("red\r\nnext");
  });

  it("preserves trailing whitespace for streaming output", () => {
    expect(stripTerminalControl("partial ")).toBe("partial ");
  });
});
