<script lang="ts">
  import {
    Bookmark,
    ChevronLeft,
    PanelLeft,
    Plus,
    Settings,
    SquareTerminal,
    X,
  } from "@lucide/svelte";
  import type { SavedSessionSummary, UiSession } from "$lib/types";

  interface Props {
    sessions: UiSession[];
    savedSessions: SavedSessionSummary[];
    activeId?: string;
    collapsed?: boolean;
    onSelect: (id: string) => void;
    onSelectSaved: (id: string) => void;
    onDeleteSaved: (id: string) => void;
    onNew: () => void;
    onClose: (id: string) => void;
    onToggle: () => void;
    onSettings: () => void;
  }

  let {
    sessions,
    savedSessions,
    activeId,
    collapsed = false,
    onSelect,
    onSelectSaved,
    onDeleteSaved,
    onNew,
    onClose,
    onToggle,
    onSettings,
  }: Props = $props();

  function statusLabel(session: UiSession): string {
    if (session.error) return "Error";
    if (session.status === "running") return "Running";
    if (session.status === "exited") return "Exited";
    return "Ready";
  }
</script>

<aside class:collapsed class="sidebar" aria-label="Terminal sessions">
  <header class="brand-row">
    <button class="brand" onclick={onToggle} aria-label={collapsed ? "Expand sessions" : "Collapse sessions"}>
      <span class="brand-mark"><SquareTerminal size={21} strokeWidth={1.8} /></span>
      {#if !collapsed}<span>Chatterm</span>{/if}
    </button>
    {#if !collapsed}
      <button class="icon-button quiet" onclick={onToggle} aria-label="Collapse sessions">
        <ChevronLeft size={18} />
      </button>
    {/if}
  </header>

  {#if collapsed}
    <nav class="collapsed-nav" aria-label="Session shortcuts">
      {#each sessions as session (session.id)}
        <button
          class:active={session.id === activeId}
          class="collapsed-session"
          onclick={() => onSelect(session.id)}
          title={session.name}
          aria-label={`Open ${session.name}`}
        >
          <SquareTerminal size={19} />
          <span class:running={session.status === "running"} class="status-dot"></span>
        </button>
      {/each}
      <button class="collapsed-session add" onclick={onNew} title="New session" aria-label="New session">
        <Plus size={19} />
      </button>
    </nav>
  {:else}
    <div class="sidebar-scroll">
      <section>
        <div class="section-label"><span>Active</span><span>{sessions.length}</span></div>
        <div class="session-list">
          {#each sessions as session (session.id)}
            <div class:active={session.id === activeId} class="session-row">
              <button class="session-main" onclick={() => onSelect(session.id)}>
                <span class="session-icon"><SquareTerminal size={17} /></span>
                <span class="session-copy">
                  <strong>{session.name}</strong>
                  <small>{session.blocks.length} commands · {statusLabel(session)}</small>
                </span>
                <span
                  class:running={session.status === "running"}
                  class:error={session.status === "error"}
                  class="status-dot"
                ></span>
              </button>
              <button
                class="session-close"
                onclick={() => onClose(session.id)}
                aria-label={`Close ${session.name}`}
                title="Close session"
              ><X size={14} /></button>
            </div>
          {/each}
        </div>
      </section>

      <section>
        <div class="section-label"><span>Saved</span><span>{savedSessions.length}</span></div>
        <div class="session-list">
          {#each savedSessions as saved (saved.id)}
            <div class="saved-session-row">
              <button class="saved-row" onclick={() => onSelectSaved(saved.id)}>
                <span class="session-icon saved"><Bookmark size={16} /></span>
                <span class="session-copy">
                  <strong>{saved.name}</strong>
                  <small>{saved.commandCount} commands · {new Date(saved.savedAt).toLocaleDateString()}</small>
                </span>
              </button>
              <button class="saved-delete" onclick={() => onDeleteSaved(saved.id)} aria-label={`Delete ${saved.name}`} title="Delete saved session"><X size={14} /></button>
            </div>
          {:else}
            <p class="saved-empty">Saved sessions stay on this device.</p>
          {/each}
        </div>
      </section>
    </div>

    <div class="sidebar-actions">
      <button class="new-session" onclick={onNew}><Plus size={17} />New session</button>
    </div>
  {/if}

  <footer class="sidebar-footer">
    <button class="icon-button" onclick={onSettings} aria-label="Open settings" title="Settings">
      <Settings size={18} />
    </button>
    {#if !collapsed}
      <span class="local-label">Local · Private</span>
      <button class="icon-button quiet" onclick={onToggle} aria-label="Collapse sessions">
        <PanelLeft size={17} />
      </button>
    {/if}
  </footer>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--sidebar);
    border-right: 1px solid var(--line);
    transition: width 180ms ease, min-width 180ms ease;
  }

  .sidebar.collapsed {
    width: 58px;
    min-width: 58px;
  }

  .brand-row,
  .sidebar-footer {
    min-height: 58px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 13px;
    border-bottom: 1px solid var(--line);
  }

  .brand {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0;
    color: var(--text-strong);
    font-size: 16px;
    font-weight: 680;
    letter-spacing: -0.025em;
    background: none;
    border: 0;
    cursor: pointer;
  }

  .brand-mark,
  .session-icon {
    display: inline-grid;
    place-items: center;
    color: var(--lavender);
  }

  .sidebar-scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 17px 10px;
  }

  section + section { margin-top: 25px; }

  .section-label {
    display: flex;
    justify-content: space-between;
    padding: 0 10px 8px;
    color: var(--text-muted);
    font-size: 10px;
    font-weight: 720;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .session-list { display: grid; gap: 3px; }

  .session-row {
    position: relative;
    display: flex;
    border-radius: 9px;
    border: 1px solid transparent;
  }

  .session-row:hover { background: var(--surface-hover); }
  .session-row.active {
    background: var(--surface-selected);
    border-color: color-mix(in srgb, var(--lavender) 18%, transparent);
  }

  .session-row.active::before {
    content: "";
    position: absolute;
    left: -1px;
    top: 9px;
    bottom: 9px;
    width: 2px;
    border-radius: 2px;
    background: var(--lavender);
  }

  .session-main,
  .saved-row {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 8px 9px 10px;
    color: inherit;
    text-align: left;
    background: transparent;
    border: 0;
    cursor: pointer;
  }

  .saved-row { border-radius: 9px; }
  .saved-row:hover { background: var(--surface-hover); }
  .saved-session-row { display: flex; border-radius: 9px; }
  .saved-session-row:hover { background: var(--surface-hover); }
  .saved-session-row .saved-row:hover { background: transparent; }
  .saved-delete { width: 29px; display: grid; place-items: center; padding: 0; color: var(--text-dim); opacity: 0; background: transparent; border: 0; border-radius: 7px; cursor: pointer; }
  .saved-session-row:hover .saved-delete, .saved-delete:focus-visible { opacity: 1; }
  .saved-delete:hover { color: var(--signal-red); }
  .session-icon.saved { color: var(--text-muted); }

  .session-copy {
    flex: 1;
    min-width: 0;
    display: grid;
    gap: 2px;
  }

  .session-copy strong {
    overflow: hidden;
    color: var(--text);
    font-size: 12.5px;
    font-weight: 590;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .session-copy small {
    overflow: hidden;
    color: var(--text-dim);
    font-size: 10.5px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status-dot {
    width: 7px;
    height: 7px;
    flex: none;
    border-radius: 50%;
    background: var(--sea-glass);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--sea-glass) 10%, transparent);
  }

  .status-dot.running { background: var(--copper); box-shadow: 0 0 0 3px color-mix(in srgb, var(--copper) 11%, transparent); }
  .status-dot.error { background: var(--signal-red); }

  .session-close {
    width: 27px;
    display: grid;
    place-items: center;
    padding: 0;
    color: var(--text-dim);
    opacity: 0;
    background: none;
    border: 0;
    cursor: pointer;
  }

  .session-row:hover .session-close,
  .session-close:focus-visible { opacity: 1; }

  .saved-empty {
    margin: 3px 10px 0;
    color: var(--text-dim);
    font-size: 11px;
    line-height: 1.5;
  }

  .sidebar-actions { padding: 10px; }
  .new-session {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    padding: 9px 12px;
    color: var(--lavender);
    font-size: 12px;
    font-weight: 620;
    background: transparent;
    border: 1px solid color-mix(in srgb, var(--lavender) 45%, var(--line));
    border-radius: 9px;
    cursor: pointer;
  }

  .new-session:hover { background: color-mix(in srgb, var(--lavender) 8%, transparent); }

  .sidebar-footer {
    border-top: 1px solid var(--line);
    border-bottom: 0;
  }

  .local-label {
    flex: 1;
    color: var(--text-dim);
    font-size: 10.5px;
  }

  .icon-button,
  .collapsed-session {
    display: grid;
    place-items: center;
    padding: 0;
    color: var(--text-muted);
    background: transparent;
    border: 0;
    border-radius: 7px;
    cursor: pointer;
  }

  .icon-button { width: 30px; height: 30px; }
  .icon-button:hover, .collapsed-session:hover { color: var(--text); background: var(--surface-hover); }
  .icon-button.quiet { width: 27px; height: 27px; }

  .collapsed-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
    padding-top: 13px;
  }

  .collapsed-session {
    position: relative;
    width: 38px;
    height: 38px;
  }

  .collapsed-session.active { color: var(--lavender); background: var(--surface-selected); }
  .collapsed-session.add { margin-top: 4px; }
  .collapsed-session .status-dot { position: absolute; right: 4px; bottom: 4px; width: 5px; height: 5px; box-shadow: none; }

  .collapsed .brand-row { justify-content: center; padding: 0; }
  .collapsed .brand { flex: none; }
  .collapsed .sidebar-footer { justify-content: center; padding: 0; }

  @media (prefers-reduced-motion: reduce) {
    .sidebar { transition: none; }
  }
</style>
