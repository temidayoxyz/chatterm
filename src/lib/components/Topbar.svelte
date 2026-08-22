<script lang="ts">
  import {
    Bookmark,
    Blocks,
    Check,
    Focus,
    MessageSquare,
    Search,
    ShieldCheck,
  } from "@lucide/svelte";
  import type { AppMode, ThreadLayout, UiSession } from "$lib/types";

  interface Props {
    session?: UiSession;
    mode: AppMode;
    layout: ThreadLayout;
    onMode: (mode: AppMode) => void;
    onLayout: (layout: ThreadLayout) => void;
    onSave: () => void;
    onSearch: () => void;
  }

  let { session, mode, layout, onMode, onLayout, onSave, onSearch }: Props = $props();
</script>

<header class="topbar">
  <div class="workspace">
    <span class="cwd" title={session?.cwd}>{session?.cwd ?? "Starting terminal…"}</span>
    {#if session}
      <span class="chip shell">❯_&nbsp; {session.shellName}</span>
      <span class="chip privacy"><ShieldCheck size={13} /> Local · Private</span>
    {/if}
  </div>

  <div class="top-actions">
    {#if mode === "thread"}
      <div class="segmented" aria-label="Thread layout">
        <button class:active={layout === "blocks"} onclick={() => onLayout("blocks")} title="Execution blocks" aria-label="Execution blocks"><Blocks size={15} /></button>
        <button class:active={layout === "chat"} onclick={() => onLayout("chat")} title="Chat layout" aria-label="Chat layout"><MessageSquare size={15} /></button>
      </div>
      <button class="mode-button" onclick={() => onMode("focus")}><Focus size={15} />Focus mode</button>
    {:else}
      <button class="mode-button active" onclick={() => onMode("thread")}><Blocks size={15} />Thread mode</button>
    {/if}
    <span class:running={session?.status === "running"} class="process-state">
      {#if session?.status === "running"}<span class="pulse"></span>Running{:else if session?.status === "starting"}Starting…{:else}<Check size={13} />Ready{/if}
    </span>
    <button class:active={session?.saved} class="icon-button" onclick={onSave} title={session?.saved ? "Session saved" : "Save session"} aria-label="Save session"><Bookmark size={16} /></button>
    <button class="icon-button" onclick={onSearch} title="Search" aria-label="Search"><Search size={16} /></button>
  </div>
</header>

<style>
  .topbar {
    height: 58px;
    min-height: 58px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding: 0 16px 0 22px;
    background: color-mix(in srgb, var(--canvas) 93%, var(--surface));
    border-bottom: 1px solid var(--line);
  }

  .workspace,
  .top-actions { display: flex; align-items: center; gap: 8px; min-width: 0; }

  .cwd {
    max-width: min(36vw, 520px);
    overflow: hidden;
    color: var(--lavender);
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 560;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chip,
  .process-state {
    height: 28px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0 9px;
    color: var(--text-muted);
    font-size: 10.5px;
    white-space: nowrap;
    border: 1px solid var(--line);
    border-radius: 7px;
  }

  .chip.shell { font-family: var(--font-mono); color: var(--text); }
  .chip.privacy { color: var(--sea-glass); }

  .segmented {
    height: 30px;
    display: inline-flex;
    padding: 2px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 8px;
  }

  .segmented button {
    width: 29px;
    display: grid;
    place-items: center;
    padding: 0;
    color: var(--text-dim);
    background: transparent;
    border: 0;
    border-radius: 5px;
    cursor: pointer;
  }

  .segmented button:hover { color: var(--text); }
  .segmented button.active { color: var(--lavender); background: var(--surface-selected); }

  .mode-button {
    height: 31px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 10px;
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 590;
    background: transparent;
    border: 1px solid var(--line);
    border-radius: 8px;
    cursor: pointer;
  }

  .mode-button:hover,
  .mode-button.active { color: var(--text); border-color: color-mix(in srgb, var(--lavender) 32%, var(--line)); }

  .process-state { border: 0; padding: 0 5px; }
  .process-state.running { color: var(--copper); }
  .pulse { width: 7px; height: 7px; border-radius: 50%; background: var(--copper); box-shadow: 0 0 0 3px color-mix(in srgb, var(--copper) 13%, transparent); }

  .icon-button {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    padding: 0;
    color: var(--text-dim);
    background: transparent;
    border: 0;
    border-radius: 7px;
    cursor: pointer;
  }

  .icon-button:hover { color: var(--text); background: var(--surface-hover); }
  .icon-button.active { color: var(--sea-glass); }

  @media (max-width: 1080px) {
    .chip.privacy, .process-state { display: none; }
  }
</style>
