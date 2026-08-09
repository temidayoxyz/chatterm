<script lang="ts">
  import { Blocks, Maximize2 } from "@lucide/svelte";
  import TerminalSurface from "./TerminalSurface.svelte";
  import type { UiSession } from "$lib/types";

  interface Props {
    session?: UiSession;
    fontSize: number;
    searchQuery?: string;
    onInput: (data: string) => void;
    onResize: (rows: number, cols: number) => void;
    onThread: () => void;
  }

  let { session, fontSize, searchQuery = "", onInput, onResize, onThread }: Props = $props();
</script>

<section class="focus-view">
  <TerminalSurface {session} {fontSize} {searchQuery} {onInput} {onResize} />
  <div class="focus-badge"><Maximize2 size={12} /><span>Focus</span></div>
  <button class="thread-return" onclick={onThread} title="Return to Thread mode"><Blocks size={14} />Thread</button>
</section>

<style>
  .focus-view { position: relative; flex: 1; min-width: 0; min-height: 0; overflow: hidden; background: var(--terminal-bg); }
  .focus-badge { position: absolute; right: 17px; bottom: 14px; display: flex; align-items: center; gap: 6px; padding: 5px 8px; color: var(--text-faint); font: 9px var(--font-mono); letter-spacing: .1em; text-transform: uppercase; pointer-events: none; background: color-mix(in srgb, var(--terminal-bg) 84%, transparent); border: 1px solid var(--line); border-radius: 6px; }
  .thread-return { position: absolute; right: 17px; top: 14px; display: flex; align-items: center; gap: 6px; padding: 7px 9px; color: var(--text-muted); font: 10px var(--font-ui); background: color-mix(in srgb, var(--terminal-bg) 88%, transparent); border: 1px solid var(--line); border-radius: 7px; opacity: 0; cursor: pointer; transition: opacity 140ms ease; }
  .focus-view:hover .thread-return, .thread-return:focus-visible { opacity: 1; }
  .thread-return:hover { color: var(--text); border-color: var(--line-strong); }
  @media (prefers-reduced-motion: reduce) { .thread-return { transition: none; } }
</style>
