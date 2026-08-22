const DSR_QUERY = "\u001b[6n";
const DSR_REPLY = "\u001b[1;1R";
const MAX_TAIL = 2 * DSR_QUERY.length;

/**
 * Answers terminal queries for shells whose output is not attached to a live
 * terminal surface. Windows PowerShell's PSReadLine asks for the cursor
 * position with ESC[6n during startup and blocks until a host replies, so a
 * shell rendered only as command threads would never accept input. A real
 * terminal emulator answers these queries itself; this stands in for one.
 */
export class TerminalQueryResponder {
  private tail = "";

  /** Feed one chunk of terminal output; returns replies to write back. */
  push(raw: string): string[] {
    const replies: string[] = [];
    this.tail += raw;
    let index = this.tail.indexOf(DSR_QUERY);
    while (index >= 0) {
      replies.push(DSR_REPLY);
      this.tail = this.tail.slice(index + DSR_QUERY.length);
      index = this.tail.indexOf(DSR_QUERY);
    }
    // A query can straddle chunk boundaries; keep enough of the stream end
    // for the next push to complete it. Partial queries are always a suffix,
    // so trimming to the query length never cuts into one.
    this.tail = this.tail.slice(-MAX_TAIL);
    return replies;
  }

  reset(): void {
    this.tail = "";
  }
}
