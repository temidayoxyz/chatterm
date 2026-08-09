<script lang="ts">
  import { Check, X } from "@lucide/svelte";
  import type { AppSettings } from "$lib/types";

  interface Props {
    settings: AppSettings;
    onChange: (settings: AppSettings) => void;
    onClose: () => void;
  }

  let { settings, onChange, onClose }: Props = $props();

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    onChange({ ...settings, [key]: value });
  }
</script>

<div class="scrim" role="presentation" onclick={(event) => event.target === event.currentTarget && onClose()}>
  <aside class="settings-panel" aria-label="Settings">
    <header>
      <div><span class="eyebrow">Preferences</span><h2>Settings</h2></div>
      <button class="close" onclick={onClose} aria-label="Close settings"><X size={18} /></button>
    </header>

    <div class="settings-body">
      <section>
        <h3>Appearance</h3>
        <p>Choose a surface without changing terminal ANSI colors.</p>
        <div class="theme-grid">
          {#each [
            { id: "basalt", name: "Basalt", colors: ["#10131b", "#222737", "#afa7ff"] },
            { id: "paper", name: "Paper", colors: ["#f1f2ee", "#ffffff", "#5f56b8"] },
            { id: "classic", name: "Classic", colors: ["#090d0b", "#101713", "#7bcf94"] },
          ] as theme}
            <button class:active={settings.theme === theme.id} class="theme-card" onclick={() => update("theme", theme.id as AppSettings["theme"])}>
              <span class="swatches">{#each theme.colors as color}<i style={`background:${color}`}></i>{/each}</span>
              <span>{theme.name}</span>
              {#if settings.theme === theme.id}<Check size={14} />{/if}
            </button>
          {/each}
        </div>

        <label class="range-row">
          <span><strong>Terminal text</strong><small>{settings.fontSize}px</small></span>
          <input type="range" min="11" max="20" step="1" value={settings.fontSize} oninput={(event) => update("fontSize", Number(event.currentTarget.value))} />
        </label>
      </section>

      <section>
        <h3>Thread mode</h3>
        <label class="setting-row">
          <span><strong>Enter Focus mode automatically</strong><small>Switch when an app opens the alternate terminal screen.</small></span>
          <input type="checkbox" checked={settings.autoFocusInteractive} onchange={(event) => update("autoFocusInteractive", event.currentTarget.checked)} />
        </label>
        <label class="setting-row">
          <span><strong>Reduce interface motion</strong><small>Keep state transitions instant and quiet.</small></span>
          <input type="checkbox" checked={settings.reducedMotion} onchange={(event) => update("reducedMotion", event.currentTarget.checked)} />
        </label>
      </section>

      <section class="privacy-note">
        <h3>Local by design</h3>
        <p>Chatterm has no account, cloud sync, analytics, telemetry, or built-in AI. Session history is written only when you choose Save session.</p>
      </section>
    </div>
  </aside>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: flex;
    justify-content: flex-end;
    background: color-mix(in srgb, #000 42%, transparent);
    backdrop-filter: blur(2px);
  }

  .settings-panel {
    width: min(440px, 92vw);
    height: 100%;
    display: flex;
    flex-direction: column;
    color: var(--text);
    background: var(--sidebar);
    border-left: 1px solid var(--line-strong);
    box-shadow: -24px 0 70px rgb(0 0 0 / 24%);
  }

  header {
    min-height: 72px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 22px;
    border-bottom: 1px solid var(--line);
  }

  .eyebrow { color: var(--lavender); font-size: 9px; font-weight: 750; letter-spacing: .16em; text-transform: uppercase; }
  h2 { margin: 2px 0 0; color: var(--text-strong); font-size: 19px; letter-spacing: -.03em; }
  .close { width: 32px; height: 32px; display: grid; place-items: center; color: var(--text-muted); background: transparent; border: 0; border-radius: 7px; cursor: pointer; }
  .close:hover { color: var(--text); background: var(--surface-hover); }

  .settings-body { overflow: auto; padding: 10px 22px 36px; }
  section { padding: 21px 0 24px; border-bottom: 1px solid var(--line); }
  h3 { margin: 0; color: var(--text); font-size: 12px; font-weight: 680; }
  section > p { margin: 5px 0 14px; color: var(--text-dim); font-size: 11px; line-height: 1.55; }

  .theme-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .theme-card { position: relative; display: grid; gap: 8px; padding: 9px; color: var(--text-muted); font-size: 10px; text-align: left; background: var(--surface); border: 1px solid var(--line); border-radius: 9px; cursor: pointer; }
  .theme-card:hover, .theme-card.active { color: var(--text); border-color: color-mix(in srgb, var(--lavender) 48%, var(--line)); }
  .theme-card > :global(svg) { position: absolute; right: 7px; bottom: 7px; color: var(--lavender); }
  .swatches { height: 48px; display: flex; overflow: hidden; border-radius: 5px; }
  .swatches i { flex: 1; }

  .range-row, .setting-row { display: flex; align-items: center; justify-content: space-between; gap: 18px; margin-top: 16px; }
  .range-row > span, .setting-row > span { display: grid; gap: 3px; }
  strong { color: var(--text); font-size: 11px; font-weight: 620; }
  small { color: var(--text-dim); font-size: 10px; line-height: 1.45; }
  input[type="range"] { width: 150px; accent-color: var(--lavender); }
  input[type="checkbox"] { position: relative; width: 32px; height: 18px; flex: none; appearance: none; background: var(--surface); border: 1px solid var(--line-strong); border-radius: 10px; cursor: pointer; transition: background 140ms ease, border-color 140ms ease; }
  input[type="checkbox"]::after { content: ""; position: absolute; top: 3px; left: 3px; width: 10px; height: 10px; background: var(--text-dim); border-radius: 50%; transition: transform 140ms ease, background 140ms ease; }
  input[type="checkbox"]:checked { background: color-mix(in srgb, var(--lavender) 28%, var(--surface)); border-color: var(--lavender); }
  input[type="checkbox"]:checked::after { background: var(--lavender); transform: translateX(14px); }

  .privacy-note { border: 0; }
  .privacy-note h3 { color: var(--sea-glass); }
</style>
