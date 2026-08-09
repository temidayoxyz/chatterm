<script lang="ts">
  import { onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import { SearchAddon } from "@xterm/addon-search";
  import "@xterm/xterm/css/xterm.css";
  import type { UiSession } from "$lib/types";

  interface Props {
    session?: UiSession;
    fontSize: number;
    searchQuery?: string;
    onInput: (data: string) => void;
    onResize: (rows: number, cols: number) => void;
  }

  let { session, fontSize, searchQuery = "", onInput, onResize }: Props = $props();
  let host: HTMLDivElement;
  let terminal: Terminal | undefined;
  let fitAddon: FitAddon | undefined;
  let searchAddon: SearchAddon | undefined;
  let writtenChunks = 0;
  let sessionId: string | undefined;

  const decoder = new TextDecoder();

  function terminalTheme() {
    const styles = getComputedStyle(document.documentElement);
    const color = (name: string, fallback: string) => styles.getPropertyValue(name).trim() || fallback;
    return {
      background: color("--terminal-bg", "#10131b"),
      foreground: color("--text", "#d8dae5"),
      cursor: color("--lavender", "#afa7ff"),
      cursorAccent: color("--terminal-bg", "#10131b"),
      selectionBackground: color("--selection", "#3b4058"),
      black: "#191c25",
      red: "#e07178",
      green: "#78cdb5",
      yellow: "#d8b475",
      blue: "#83a8ec",
      magenta: "#b9a5ec",
      cyan: "#72c5d2",
      white: "#d8dae5",
      brightBlack: "#73798b",
      brightRed: "#f18b91",
      brightGreen: "#92dcc8",
      brightYellow: "#e6c78f",
      brightBlue: "#9abcf5",
      brightMagenta: "#cbb9f7",
      brightCyan: "#8bd6e0",
      brightWhite: "#f5f5f7",
    };
  }

  function fit() {
    if (!terminal || !fitAddon || !host.clientWidth || !host.clientHeight) return;
    try {
      fitAddon.fit();
      onResize(terminal.rows, terminal.cols);
    } catch {
      // The surface can be briefly dimensionless during a mode transition.
    }
  }

  function replay(next: UiSession | undefined) {
    if (!terminal || !next) return;
    if (sessionId !== next.id) {
      terminal.reset();
      sessionId = next.id;
      writtenChunks = 0;
      decoder.decode();
    }
    while (writtenChunks < next.rawChunks.length) {
      terminal.write(decoder.decode(next.rawChunks[writtenChunks], { stream: true }));
      writtenChunks += 1;
    }
  }

  onMount(() => {
    terminal = new Terminal({
      allowProposedApi: false,
      allowTransparency: false,
      convertEol: false,
      cursorBlink: true,
      cursorStyle: "bar",
      cursorWidth: 2,
      disableStdin: Boolean(session?.readOnly),
      drawBoldTextInBrightColors: false,
      fontFamily: "var(--font-mono)",
      fontSize,
      fontWeight: "400",
      fontWeightBold: "600",
      letterSpacing: 0,
      lineHeight: 1.25,
      minimumContrastRatio: 4.5,
      scrollback: 10_000,
      smoothScrollDuration: 0,
      theme: terminalTheme(),
      windowOptions: {},
    });
    fitAddon = new FitAddon();
    searchAddon = new SearchAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(searchAddon);
    terminal.open(host);
    terminal.onData(onInput);
    terminal.attachCustomKeyEventHandler((event) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "v") return false;
      return true;
    });

    const observer = new ResizeObserver(fit);
    observer.observe(host);
    replay(session);
    requestAnimationFrame(() => {
      fit();
      terminal?.focus();
    });

    return () => {
      observer.disconnect();
      terminal?.dispose();
      terminal = undefined;
    };
  });

  $effect(() => {
    session?.rawChunks.length;
    replay(session);
  });

  $effect(() => {
    if (!terminal) return;
    terminal.options.fontSize = fontSize;
    terminal.options.theme = terminalTheme();
    requestAnimationFrame(fit);
  });

  $effect(() => {
    if (!searchAddon || !searchQuery) return;
    searchAddon.findNext(searchQuery, { incremental: true });
  });
</script>

<div class="terminal-shell">
  <div bind:this={host} class="terminal-host" aria-label="Interactive terminal"></div>
</div>

<style>
  .terminal-shell,
  .terminal-host {
    width: 100%;
    height: 100%;
    min-width: 0;
    min-height: 0;
  }

  .terminal-shell {
    padding: 18px 20px 12px;
    background: var(--terminal-bg);
  }

  .terminal-host :global(.xterm) { height: 100%; }
  .terminal-host :global(.xterm-viewport) { scrollbar-color: var(--line-strong) transparent; }
  .terminal-host :global(.xterm-screen) { outline: none; }
</style>
