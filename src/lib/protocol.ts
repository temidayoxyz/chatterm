export type ProtocolEvent =
  | { type: "promptStart"; cwd?: string }
  | { type: "promptEnd" }
  | { type: "commandStart"; command?: string }
  | { type: "commandEnd"; exitCode?: number };

export type ProtocolPart =
  | { type: "text"; value: string }
  | { type: "event"; value: ProtocolEvent };

const PREFIX = "\u001b]777;";
const BEL = "\u0007";
const ST = "\u001b\\";

export class ChattermProtocolParser {
  private buffer = "";

  push(input: string): { visible: string; events: ProtocolEvent[]; parts: ProtocolPart[] } {
    this.buffer += input;
    let visible = "";
    const events: ProtocolEvent[] = [];
    const parts: ProtocolPart[] = [];

    const appendText = (value: string) => {
      if (!value) return;
      visible += value;
      const previous = parts.at(-1);
      if (previous?.type === "text") previous.value += value;
      else parts.push({ type: "text", value });
    };

    while (this.buffer.length > 0) {
      const start = this.buffer.indexOf(PREFIX);
      if (start < 0) {
        const keep = suffixPrefixLength(this.buffer, PREFIX);
        appendText(this.buffer.slice(0, this.buffer.length - keep));
        this.buffer = this.buffer.slice(this.buffer.length - keep);
        break;
      }

      appendText(this.buffer.slice(0, start));
      const payloadStart = start + PREFIX.length;
      const belIndex = this.buffer.indexOf(BEL, payloadStart);
      const stIndex = this.buffer.indexOf(ST, payloadStart);
      const candidates = [belIndex, stIndex].filter((index) => index >= 0);
      if (candidates.length === 0) {
        this.buffer = this.buffer.slice(start);
        break;
      }

      const end = Math.min(...candidates);
      const terminatorLength = end === stIndex ? ST.length : BEL.length;
      const payload = this.buffer.slice(payloadStart, end);
      const event = parsePayload(payload);
      if (event) {
        events.push(event);
        parts.push({ type: "event", value: event });
      }
      this.buffer = this.buffer.slice(end + terminatorLength);
    }

    return { visible, events, parts };
  }

  reset(): void {
    this.buffer = "";
  }
}

function parsePayload(payload: string): ProtocolEvent | undefined {
  const [kind, ...rest] = payload.split(";");
  switch (kind) {
    case "A":
      return { type: "promptStart", cwd: rest.join(";") || undefined };
    case "B":
      return { type: "promptEnd" };
    case "C":
      return { type: "commandStart", command: rest.join(";") || undefined };
    case "D": {
      const code = Number.parseInt(rest[0] ?? "", 10);
      return { type: "commandEnd", exitCode: Number.isFinite(code) ? code : undefined };
    }
    default:
      return undefined;
  }
}

function suffixPrefixLength(value: string, prefix: string): number {
  const limit = Math.min(value.length, prefix.length - 1);
  for (let length = limit; length > 0; length -= 1) {
    if (prefix.startsWith(value.slice(-length))) return length;
  }
  return 0;
}

export function stripTerminalControl(value: string): string {
  return value
    .replace(/\u001b\][\s\S]*?(?:\u0007|\u001b\\)/g, "")
    .replace(/\u001bP[\s\S]*?\u001b\\/g, "")
    .replace(/\u001b\[[0-?]*[ -/]*[@-~]/g, "")
    .replace(/\u001b[@-_]/g, "")
    .replace(/\r(?!\n)/g, "\n")
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001a\u001c-\u001f\u007f]/g, "")
    .replace(/\n{4,}/g, "\n\n\n");
}
