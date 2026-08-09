<script lang="ts">
  import { tick } from "svelte";
  import {
    Check,
    ChevronRight,
    CircleStop,
    Clock3,
    Copy,
    CornerDownLeft,
    LoaderCircle,
    RotateCcw,
    Sparkles,
  } from "@lucide/svelte";
  import type { CommandBlock, ThreadLayout, UiSession } from "$lib/types";

  interface Props {
    session?: UiSession;
    layout: ThreadLayout;
    searchQuery?: string;
    onSubmit: (command: string) => void;
    onInterrupt: () => void;
  }

  let { session, layout, searchQuery = "", onSubmit, onInterrupt }: Props = $props();
  let command = $state("");
  let composer = $state<HTMLTextAreaElement>();
  let viewport = $state<HTMLDivElement>();
  let visibleBlocks = $derived(
    !searchQuery.trim()
      ? session?.blocks ?? []
      : (session?.blocks ?? []).filter((block) => `${block.command}\n${block.output}`.toLowerCase().includes(searchQuery.trim().toLowerCase())),
  );

  function submit() {
    const value = command.trim();
    if (!value || !session || session.readOnly || session.status === "running" || session.status === "exited") return;
    onSubmit(value);
    command = "";
    tick().then(() => viewport?.scrollTo({ top: viewport.scrollHeight, behavior: "smooth" }));
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      submit();
    }
  }

  function duration(block: CommandBlock): string {
    if (!block.endedAt) return "running";
    const elapsed = new Date(block.endedAt).getTime() - new Date(block.startedAt).getTime();
    return elapsed < 1_000 ? `${elapsed}ms` : `${(elapsed / 1_000).toFixed(1)}s`;
  }

  async function copy(value: string) {
    await navigator.clipboard.writeText(value);
  }
</script>

<section class:chat={layout === "chat"} class="thread-view">
  <div bind:this={viewport} class="thread-scroll">
    {#if !session}
      <div class="empty-state"><LoaderCircle class="spin" size={22} /><span>Opening your shell…</span></div>
    {:else if session.blocks.length === 0}
      <div class="thread-intro">
        <div class="intro-mark"><Sparkles size={18} /></div>
        <span class="eyebrow">A quieter command line</span>
        <h1>Your terminal, with a memory.</h1>
        <p>Run a command below. Chatterm will keep the input, output, status, and timing together—without sending any of it away.</p>
        <div class="suggestions">
          {#each ["git status", "npm run build", "ls -la"] as suggestion}
            <button onclick={() => { command = suggestion; composer?.focus(); }}><ChevronRight size={14} />{suggestion}</button>
          {/each}
        </div>
      </div>
    {:else}
      <div class="execution-thread">
        {#each visibleBlocks as block, index (block.id)}
          <article class:failed={block.status === "failed"} class:running={block.status === "running"} class="command-block">
            <div class="rail" aria-hidden="true">
              <span class="rail-index">{String(index + 1).padStart(2, "0")}</span>
              <span class="rail-node">
                {#if block.status === "running"}<LoaderCircle class="spin" size={13} />
                {:else if block.status === "success"}<Check size={13} />
                {:else}<CircleStop size={12} />{/if}
              </span>
              <span class="rail-line"></span>
            </div>

            <div class="block-content">
              <header class="command-header">
                <div class="command-line"><span class="prompt">❯</span><code>{block.command}</code></div>
                <div class="command-actions">
                  <span class="duration"><Clock3 size={12} />{duration(block)}</span>
                  <button onclick={() => copy(block.command)} aria-label="Copy command" title="Copy command"><Copy size={14} /></button>
                  <button onclick={() => { command = block.command; composer?.focus(); }} aria-label="Run again" title="Run again"><RotateCcw size={14} /></button>
                </div>
              </header>

              {#if block.output}
                <pre class="output">{block.output}</pre>
              {:else if block.status === "running"}
                <div class="waiting"><span></span>Waiting for output</div>
              {/if}

              <footer class="block-footer">
                <span class:bad={block.status === "failed"} class="exit-status">
                  {#if block.status === "running"}process active
                  {:else if block.status === "success"}exit {block.exitCode ?? 0}
                  {:else}{block.status}{block.exitCode !== undefined ? ` · exit ${block.exitCode}` : ""}{/if}
                </span>
                {#if block.output}<button onclick={() => copy(block.output)}><Copy size={12} />Copy output</button>{/if}
              </footer>
            </div>
          </article>
        {/each}
        {#if visibleBlocks.length === 0 && searchQuery.trim()}
          <div class="no-results">No command or output matches “{searchQuery.trim()}”.</div>
        {/if}
      </div>
    {/if}
  </div>

  {#if session && !session.readOnly}
    <div class="composer-wrap">
      <div class:busy={session.status === "running"} class="composer">
        <span class="composer-prompt">❯</span>
        <textarea
          bind:this={composer}
          bind:value={command}
          onkeydown={keydown}
          rows="1"
          spellcheck="false"
          aria-label="Terminal command"
          placeholder={session.status === "running" ? "You can queue another command…" : "Type a command…"}
        ></textarea>
        {#if session.status === "running"}
          <button class="interrupt" onclick={onInterrupt} title="Interrupt process" aria-label="Interrupt process"><CircleStop size={16} /></button>
        {:else}
          <button class="submit" disabled={!command.trim()} onclick={submit} title="Run command" aria-label="Run command"><CornerDownLeft size={16} /></button>
        {/if}
      </div>
      <div class="composer-meta"><span>{session.shellName}</span><span>Enter to run · Shift + Enter for a new line</span></div>
    </div>
  {:else if session?.readOnly}
    <div class="readonly-bar">Saved session · read only</div>
  {/if}
</section>

<style>
  .thread-view { position: relative; flex: 1; min-height: 0; display: flex; flex-direction: column; background: var(--canvas); }
  .thread-scroll { flex: 1; min-height: 0; overflow: auto; scroll-behavior: smooth; scrollbar-color: var(--line-strong) transparent; }
  .empty-state, .thread-intro { min-height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--text-muted); }
  .empty-state { gap: 10px; font-size: 12px; }
  .thread-intro { max-width: 620px; min-height: calc(100% - 40px); margin: auto; padding: 50px 28px 110px; text-align: center; }
  .intro-mark { width: 42px; height: 42px; display: grid; place-items: center; margin-bottom: 17px; color: var(--lavender); background: color-mix(in srgb, var(--lavender) 10%, var(--surface)); border: 1px solid color-mix(in srgb, var(--lavender) 26%, var(--line)); border-radius: 13px; transform: rotate(-4deg); }
  .eyebrow { color: var(--lavender); font-size: 9px; font-weight: 760; letter-spacing: .18em; text-transform: uppercase; }
  h1 { margin: 9px 0 10px; color: var(--text-strong); font-family: var(--font-display); font-size: clamp(28px, 4vw, 45px); font-weight: 520; letter-spacing: -.052em; line-height: 1.02; }
  .thread-intro p { max-width: 500px; margin: 0; color: var(--text-muted); font-size: 12.5px; line-height: 1.7; }
  .suggestions { display: flex; flex-wrap: wrap; justify-content: center; gap: 7px; margin-top: 23px; }
  .suggestions button { display: inline-flex; align-items: center; gap: 3px; padding: 7px 10px; color: var(--text-muted); font-family: var(--font-mono); font-size: 10.5px; background: var(--surface); border: 1px solid var(--line); border-radius: 7px; cursor: pointer; }
  .suggestions button:hover { color: var(--text); border-color: var(--line-strong); }
  .execution-thread { width: min(100%, 1120px); margin: 0 auto; padding: 34px 42px 150px; }
  .command-block { position: relative; display: grid; grid-template-columns: 50px 1fr; }
  .rail { position: relative; display: flex; flex-direction: column; align-items: center; }
  .rail-index { height: 22px; color: var(--text-faint); font-family: var(--font-mono); font-size: 9px; letter-spacing: .08em; }
  .rail-node { z-index: 1; width: 24px; height: 24px; display: grid; place-items: center; color: var(--sea-glass); background: var(--canvas); border: 1px solid color-mix(in srgb, var(--sea-glass) 35%, var(--line)); border-radius: 50%; }
  .failed .rail-node { color: var(--signal-red); border-color: color-mix(in srgb, var(--signal-red) 40%, var(--line)); }
  .running .rail-node { color: var(--copper); border-color: color-mix(in srgb, var(--copper) 45%, var(--line)); }
  .rail-line { width: 1px; flex: 1; min-height: 24px; background: linear-gradient(var(--line-strong), var(--line)); }
  .command-block:last-child .rail-line { background: linear-gradient(var(--line-strong), transparent); }
  .block-content { min-width: 0; margin: 0 0 19px 3px; overflow: hidden; background: var(--block); border: 1px solid var(--line); border-radius: 11px; box-shadow: 0 8px 24px rgb(0 0 0 / 6%); }
  .command-header { min-height: 50px; display: flex; align-items: center; justify-content: space-between; gap: 15px; padding: 0 13px 0 17px; background: var(--block-head); border-bottom: 1px solid var(--line); }
  .command-line { min-width: 0; display: flex; align-items: center; gap: 10px; overflow: hidden; }
  .command-line .prompt { color: var(--lavender); font: 600 14px var(--font-mono); }
  .command-line code { overflow: hidden; color: var(--text-strong); font: 500 12.5px/1.5 var(--font-mono); text-overflow: ellipsis; white-space: nowrap; }
  .command-actions { display: flex; align-items: center; gap: 2px; }
  .duration { display: inline-flex; align-items: center; gap: 5px; margin-right: 5px; color: var(--text-dim); font: 10px var(--font-mono); }
  .command-actions button, .block-footer button { display: inline-flex; align-items: center; gap: 5px; padding: 6px; color: var(--text-dim); background: transparent; border: 0; border-radius: 6px; cursor: pointer; }
  .command-actions button:hover, .block-footer button:hover { color: var(--text); background: var(--surface-hover); }
  .output { max-height: 440px; margin: 0; overflow: auto; padding: 15px 17px 17px; color: var(--terminal-text); font: 400 var(--thread-font-size, 12px)/1.62 var(--font-mono); tab-size: 4; white-space: pre-wrap; overflow-wrap: anywhere; }
  .waiting { display: flex; align-items: center; gap: 8px; padding: 16px 17px; color: var(--text-dim); font: 10.5px var(--font-mono); }
  .waiting span { width: 5px; height: 5px; border-radius: 50%; background: var(--copper); box-shadow: 9px 0 var(--copper), 18px 0 color-mix(in srgb, var(--copper) 45%, transparent); }
  .block-footer { min-height: 30px; display: flex; align-items: center; justify-content: space-between; padding: 0 10px 0 17px; border-top: 1px solid color-mix(in srgb, var(--line) 70%, transparent); }
  .exit-status { color: var(--sea-glass); font: 9.5px var(--font-mono); text-transform: uppercase; letter-spacing: .06em; }
  .exit-status.bad { color: var(--signal-red); }
  .block-footer button { font-size: 9.5px; }
  .composer-wrap { position: absolute; right: 0; bottom: 0; left: 0; z-index: 4; padding: 42px max(24px, calc((100% - 1000px) / 2)) 15px; background: linear-gradient(transparent, var(--canvas) 33%); }
  .composer { min-height: 50px; display: flex; align-items: flex-start; gap: 10px; padding: 13px 12px 11px 15px; background: var(--composer); border: 1px solid var(--line-strong); border-radius: 11px; box-shadow: 0 15px 45px rgb(0 0 0 / 18%); }
  .composer:focus-within { border-color: color-mix(in srgb, var(--lavender) 56%, var(--line)); box-shadow: 0 0 0 3px color-mix(in srgb, var(--lavender) 8%, transparent), 0 15px 45px rgb(0 0 0 / 18%); }
  .composer-prompt { padding-top: 1px; color: var(--lavender); font: 650 14px var(--font-mono); }
  textarea { flex: 1; min-width: 0; max-height: 120px; resize: none; overflow: auto; padding: 0; color: var(--text-strong); caret-color: var(--lavender); font: 12.5px/1.55 var(--font-mono); background: transparent; border: 0; outline: 0 !important; }
  textarea::placeholder { color: var(--text-faint); }
  .submit, .interrupt { width: 29px; height: 27px; display: grid; place-items: center; padding: 0; color: var(--canvas); background: var(--lavender); border: 0; border-radius: 7px; cursor: pointer; }
  .submit:disabled { opacity: .3; cursor: default; }
  .interrupt { color: var(--signal-red); background: color-mix(in srgb, var(--signal-red) 12%, var(--surface)); }
  .composer-meta { display: flex; justify-content: space-between; padding: 6px 3px 0; color: var(--text-faint); font: 9.5px var(--font-mono); }
  .readonly-bar { padding: 10px; color: var(--text-dim); font-size: 10px; text-align: center; background: var(--surface); border-top: 1px solid var(--line); }
  .no-results { margin: 80px auto; color: var(--text-dim); font: 11px var(--font-mono); text-align: center; }
  .chat .execution-thread { max-width: 900px; }
  .chat .command-block { grid-template-columns: 1fr; margin-bottom: 20px; }
  .chat .rail { display: none; }
  .chat .block-content { margin-left: 74px; border-radius: 13px 13px 13px 4px; }
  .chat .command-header { width: fit-content; min-width: min(78%, 650px); margin-left: auto; border: 1px solid color-mix(in srgb, var(--lavender) 24%, var(--line)); border-radius: 13px 13px 4px 13px; transform: translateY(-11px); background: color-mix(in srgb, var(--lavender) 7%, var(--block-head)); }
  .chat .output { padding-top: 5px; }
  :global(.spin) { animation: spin 1.1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  @media (max-width: 760px) {
    .execution-thread { padding: 24px 13px 145px; }
    .command-block { grid-template-columns: 34px 1fr; }
    .rail-index { display: none; }
    .command-actions .duration { display: none; }
    .composer-wrap { padding-right: 12px; padding-left: 12px; }
    .composer-meta span:last-child { display: none; }
    .chat .block-content { margin-left: 18px; }
  }
  @media (prefers-reduced-motion: reduce) { :global(.spin) { animation: none; } .thread-scroll { scroll-behavior: auto; } }
</style>
