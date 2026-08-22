import { describe, expect, it } from "vitest";
import { TerminalQueryResponder } from "./terminal-host";

const DSR_QUERY = "\u001b[6n";
const DSR_REPLY = "\u001b[1;1R";

describe("TerminalQueryResponder", () => {
  it("answers a complete cursor position request", () => {
    const responder = new TerminalQueryResponder();
    expect(responder.push(`noise${DSR_QUERY}noise`)).toEqual([DSR_REPLY]);
    expect(responder.push("more output")).toEqual([]);
  });

  it("answers a request split across chunks", () => {
    const responder = new TerminalQueryResponder();
    expect(responder.push("\u001b[")).toEqual([]);
    expect(responder.push("6")).toEqual([]);
    expect(responder.push("n")).toEqual([DSR_REPLY]);
  });

  it("answers repeated requests", () => {
    const responder = new TerminalQueryResponder();
    expect(responder.push(DSR_QUERY + DSR_QUERY)).toEqual([DSR_REPLY, DSR_REPLY]);
    expect(responder.push(`tail${DSR_QUERY}`)).toEqual([DSR_REPLY]);
  });

  it("keeps a partial request that trails a long chunk", () => {
    const responder = new TerminalQueryResponder();
    expect(responder.push(`x`.repeat(64) + "\u001b[6")).toEqual([]);
    expect(responder.push("n")).toEqual([DSR_REPLY]);
  });

  it("does not match cursor reports or mode-set sequences", () => {
    const responder = new TerminalQueryResponder();
    expect(responder.push("\u001b[1;1R")).toEqual([]);
    expect(responder.push("\u001b[?9001h\u001b[?1004h")).toEqual([]);
    expect(responder.push("\u001b[0n")).toEqual([]);
  });

  it("forgets buffered bytes after reset", () => {
    const responder = new TerminalQueryResponder();
    responder.push("\u001b[6");
    responder.reset();
    expect(responder.push("n")).toEqual([]);
    expect(responder.push(DSR_QUERY)).toEqual([DSR_REPLY]);
  });
});
