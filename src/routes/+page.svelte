<script lang="ts">
  import { onMount } from "svelte";
  import { Search, X } from "@lucide/svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Topbar from "$lib/components/Topbar.svelte";
  import ThreadView from "$lib/components/ThreadView.svelte";
  import FocusView from "$lib/components/FocusView.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import {
    closeSession,
    createSession,
    deleteSavedSession,
    listSavedSessions,
    listShellProfiles,
    loadSavedSession,
    resizeSession,
    saveSession,
    writeToSession,
  } from "$lib/api";
  import { ChattermProtocolParser, stripTerminalControl, type ProtocolEvent } from "$lib/protocol";
  import { TerminalQueryResponder } from "$lib/terminal-host";
  import type {
    AppMode,
    AppSettings,
    CommandBlock,
    SavedSessionSummary,
    SessionEvent,
    UiSession,
  } from "$lib/types";

  const MAX_RAW_BYTES = 12 * 1024 * 1024;
  const defaults: AppSettings = {
    theme: "basalt",
    fontSize: 13,
    threadLayout: "blocks",
    autoFocusInteractive: true,
    reducedMotion: false,
  };

  let sessions = $state<UiSession[]>([]);
  let savedSessions = $state<SavedSessionSummary[]>([]);
  let activeId = $state<string>();
  let mode = $state<AppMode>("thread");
  let settings = $state<AppSettings>(defaults);
  let sidebarCollapsed = $state(false);
  let settingsOpen = $state(false);
  let searchOpen = $state(false);
  let searchQuery = $state("");
  let autoFocused = $state(false);
  let bootError = $state<string>();

  const parsers = new Map<string, ChattermProtocolParser>();
  const decoders = new Map<string, TextDecoder>();
  const responders = new Map<string, TerminalQueryResponder>();
  const liveSurfaces = new Set<string>();
  let activeSession = $derived(sessions.find((session) => session.id === activeId));

  function updateSession(id: string, updater: (session: UiSession) => UiSession) {
    sessions = sessions.map((session) => session.id === id ? updater(session) : session);
  }

  function makeBlock(command: string): CommandBlock {
    return {
      id: crypto.randomUUID(),
      command,
      output: "",
      status: "running",
      startedAt: new Date().toISOString(),
    };
  }

  function withoutShellEcho(output: string, command: string): string {
    const lines = output.split("\n");
    const firstContent = lines.findIndex((line) => line.trim().length > 0);
    if (firstContent >= 0 && lines[firstContent].trim() === command.trim()) lines.splice(firstContent, 1);
    return lines.join("\n").replace(/^\s*\n/, "");
  }

  function appendRaw(id: string, text: string) {
    if (!text) return;
    const bytes = new TextEncoder().encode(text);
    updateSession(id, (session) => {
      let chunks = [...session.rawChunks, bytes];
      let rawBytes = session.rawBytes + bytes.byteLength;
      while (rawBytes > MAX_RAW_BYTES && chunks.length > 1) {
        const removed = chunks.shift();
        rawBytes -= removed?.byteLength ?? 0;
      }
      return { ...session, rawChunks: chunks, rawBytes };
    });
  }

  function appendBlockOutput(id: string, text: string) {
    const clean = stripTerminalControl(text);
    if (!clean) return;
    updateSession(id, (session) => {
      if (!session.currentBlockId) return session;
      const blocks = session.blocks.map((block) => block.id === session.currentBlockId
        ? { ...block, output: `${block.output}${clean}` }
        : block);
      return { ...session, blocks };
    });
  }

  function handleProtocolEvent(id: string, event: ProtocolEvent) {
    updateSession(id, (session) => {
      if (event.type === "promptStart" && event.cwd) {
        return {
          ...session,
          cwd: event.cwd,
          name: event.cwd.split(/[\\/]/).filter(Boolean).at(-1) || session.name,
        };
      }
      if (event.type === "commandStart") {
        if (session.currentBlockId) {
          const blocks = event.command
            ? session.blocks.map((block) => block.id === session.currentBlockId ? { ...block, command: event.command ?? block.command } : block)
            : session.blocks;
          return { ...session, blocks, status: "running" };
        }
        const block = makeBlock(event.command ?? "interactive command");
        return { ...session, blocks: [...session.blocks, block], currentBlockId: block.id, status: "running", saved: false };
      }
      if (event.type === "commandEnd") {
        const endedAt = new Date().toISOString();
        const blocks = session.blocks.map((block) => block.id === session.currentBlockId
          ? {
              ...block,
              output: withoutShellEcho(block.output, block.command),
              status: event.exitCode === 0 ? "success" as const : "failed" as const,
              exitCode: event.exitCode,
              endedAt,
            }
          : block);
        return { ...session, blocks, currentBlockId: undefined, status: "ready" };
      }
      if (event.type === "promptEnd" && session.status === "starting") return { ...session, status: "ready" };
      return session;
    });
  }

  function detectInteractive(id: string, text: string) {
    if (!settings.autoFocusInteractive || id !== activeId) return;
    if (/\u001b\[\?(?:47|1047|1049)h/.test(text)) {
      mode = "focus";
      autoFocused = true;
    } else if (autoFocused && /\u001b\[\?(?:47|1047|1049)l/.test(text)) {
      mode = activeSession?.enhancedThreads === false ? "focus" : "thread";
      autoFocused = false;
    }
  }

  function surfaceActive(id: string, active: boolean) {
    if (active) liveSurfaces.add(id);
    else liveSurfaces.delete(id);
  }

  function answerTerminalQueries(id: string, raw: string) {
    if (liveSurfaces.has(id)) {
      // A live terminal surface answers these queries itself.
      responders.get(id)?.reset();
      return;
    }
    const responder = responders.get(id) ?? new TerminalQueryResponder();
    responders.set(id, responder);
    for (const reply of responder.push(raw)) {
      writeToSession(id, reply).catch((error) => {
        console.error("Unable to answer a terminal query", error);
      });
    }
  }

  function handleSessionEvent(event: SessionEvent) {
    if (event.event === "data") {
      const decoder = decoders.get(event.sessionId) ?? new TextDecoder();
      decoders.set(event.sessionId, decoder);
      const raw = decoder.decode(new Uint8Array(event.data), { stream: true });
      answerTerminalQueries(event.sessionId, raw);
      detectInteractive(event.sessionId, raw);
      const parser = parsers.get(event.sessionId) ?? new ChattermProtocolParser();
      parsers.set(event.sessionId, parser);
      const result = parser.push(raw);
      for (const part of result.parts) {
        if (part.type === "text") {
          appendRaw(event.sessionId, part.value);
          appendBlockOutput(event.sessionId, part.value);
        } else {
          handleProtocolEvent(event.sessionId, part.value);
        }
      }
      return;
    }

    if (event.event === "exit") {
      updateSession(event.sessionId, (session) => ({
        ...session,
        status: "exited",
        currentBlockId: undefined,
        blocks: session.blocks.map((block) => block.status === "running"
          ? { ...block, status: event.code === 0 ? "success" : "failed", exitCode: event.code, endedAt: new Date().toISOString() }
          : block),
      }));
      return;
    }

    updateSession(event.sessionId, (session) => ({ ...session, status: "error", error: event.message }));
  }

  async function newSession() {
    try {
      bootError = undefined;
      const profiles = await listShellProfiles();
      const info = await createSession({ shellId: profiles[0]?.id }, handleSessionEvent);
      const session: UiSession = {
        ...info,
        name: info.cwd.split(/[\\/]/).filter(Boolean).at(-1) || info.shellName,
        blocks: [],
        rawChunks: [],
        rawBytes: 0,
        status: "starting",
        saved: false,
        readOnly: false,
      };
      parsers.set(info.id, new ChattermProtocolParser());
      decoders.set(info.id, new TextDecoder());
      sessions = [...sessions, session];
      activeId = info.id;
      mode = info.enhancedThreads ? "thread" : "focus";
    } catch (error) {
      bootError = error instanceof Error ? error.message : String(error);
    }
  }

  async function close(id: string) {
    const session = sessions.find((item) => item.id === id);
    if (session && !session.readOnly) await closeSession(id).catch(() => undefined);
    parsers.delete(id);
    decoders.delete(id);
    responders.delete(id);
    liveSurfaces.delete(id);
    sessions = sessions.filter((session) => session.id !== id);
    if (activeId === id) activeId = sessions.at(-1)?.id;
    if (sessions.length === 0) await newSession();
  }

  function selectSession(id: string) {
    activeId = id;
    if (sessions.find((session) => session.id === id)?.enhancedThreads === false) mode = "focus";
    autoFocused = false;
  }

  async function submit(command: string) {
    if (!activeSession || activeSession.readOnly || activeSession.status === "running") return;
    const block = makeBlock(command);
    updateSession(activeSession.id, (session) => ({
      ...session,
      blocks: [...session.blocks, block],
      currentBlockId: block.id,
      status: "running",
      saved: false,
    }));
    try {
      await writeToSession(activeSession.id, `${command}\r`);
    } catch (error) {
      updateSession(activeSession.id, (session) => ({ ...session, status: "error", error: String(error) }));
    }
  }

  async function interrupt() {
    if (activeSession && !activeSession.readOnly) await writeToSession(activeSession.id, "\u0003");
  }

  async function persistActive() {
    if (!activeSession) return;
    const summary = await saveSession({
      id: activeSession.id,
      name: activeSession.name,
      shellName: activeSession.shellName,
      cwd: activeSession.cwd,
      createdAt: activeSession.createdAt,
      blocks: activeSession.blocks,
    });
    savedSessions = [summary, ...savedSessions.filter((item) => item.id !== summary.id)];
    updateSession(activeSession.id, (session) => ({ ...session, saved: true }));
  }

  async function openSaved(id: string) {
    const existing = sessions.find((session) => session.id === `saved:${id}`);
    if (existing) {
      activeId = existing.id;
      mode = "thread";
      return;
    }
    const saved = await loadSavedSession(id);
    const session: UiSession = {
      id: `saved:${saved.id}`,
      shellId: "saved",
      shellName: saved.shellName,
      enhancedThreads: true,
      cwd: saved.cwd,
      createdAt: saved.createdAt,
      name: saved.name,
      blocks: saved.blocks,
      rawChunks: [],
      rawBytes: 0,
      status: "saved",
      saved: true,
      readOnly: true,
    };
    sessions = [...sessions, session];
    activeId = session.id;
    mode = "thread";
  }

  async function removeSaved(id: string) {
    await deleteSavedSession(id);
    savedSessions = savedSessions.filter((session) => session.id !== id);
    const openId = `saved:${id}`;
    sessions = sessions.filter((session) => session.id !== openId);
    if (activeId === openId) activeId = sessions.at(-1)?.id;
  }

  function changeSettings(next: AppSettings) {
    settings = next;
    localStorage.setItem("chatterm:settings", JSON.stringify(next));
  }

  function keydown(event: KeyboardEvent) {
    const modifier = event.ctrlKey || event.metaKey;
    if (modifier && event.key.toLowerCase() === "n") { event.preventDefault(); newSession(); }
    if (modifier && event.key.toLowerCase() === "s") { event.preventDefault(); persistActive(); }
    if (modifier && event.key.toLowerCase() === "f") { event.preventDefault(); searchOpen = true; }
    if (modifier && event.key === "Enter") {
      event.preventDefault();
      mode = mode === "focus" && activeSession?.enhancedThreads !== false ? "thread" : "focus";
    }
    if (event.key === "Escape" && searchOpen) searchOpen = false;
  }

  onMount(() => {
    const stored = localStorage.getItem("chatterm:settings");
    if (stored) {
      try { settings = { ...defaults, ...JSON.parse(stored) }; } catch { settings = defaults; }
    }
    listSavedSessions().then((items) => savedSessions = items).catch(() => undefined);
    newSession();
    window.addEventListener("keydown", keydown);
    return () => {
      window.removeEventListener("keydown", keydown);
      for (const session of sessions) if (!session.readOnly) closeSession(session.id).catch(() => undefined);
    };
  });
</script>

<svelte:head><title>Chatterm — Your terminal, with a memory</title></svelte:head>

<div
  class:reduce-motion={settings.reducedMotion}
  class="app-shell theme-{settings.theme}"
  style={`--thread-font-size:${Math.max(11, settings.fontSize - 1)}px`}
>
  <Sidebar
    {sessions}
    {savedSessions}
    {activeId}
    collapsed={sidebarCollapsed}
    onSelect={selectSession}
    onSelectSaved={openSaved}
    onDeleteSaved={removeSaved}
    onNew={newSession}
    onClose={close}
    onToggle={() => sidebarCollapsed = !sidebarCollapsed}
    onSettings={() => settingsOpen = true}
  />

  <main class="workspace-shell">
    <Topbar
      session={activeSession}
      {mode}
      layout={settings.threadLayout}
      onMode={(next) => { mode = next === "thread" && activeSession?.enhancedThreads === false ? "focus" : next; autoFocused = false; }}
      onLayout={(threadLayout) => changeSettings({ ...settings, threadLayout })}
      onSave={persistActive}
      onSearch={() => searchOpen = !searchOpen}
    />

    {#if bootError && !activeSession}
      <section class="fatal-state">
        <span>Terminal unavailable</span><strong>{bootError}</strong><button onclick={newSession}>Try again</button>
      </section>
    {:else if mode === "thread"}
      <ThreadView session={activeSession} layout={settings.threadLayout} {searchQuery} onSubmit={submit} onInterrupt={interrupt} />
    {:else}
      <FocusView
        session={activeSession}
        fontSize={settings.fontSize}
        {searchQuery}
        onInput={(data) => activeSession && !activeSession.readOnly && writeToSession(activeSession.id, data)}
        onResize={(rows, cols) => activeSession && !activeSession.readOnly && resizeSession(activeSession.id, rows, cols)}
        onSurfaceActive={surfaceActive}
        onThread={() => mode = activeSession?.enhancedThreads === false ? "focus" : "thread"}
      />
    {/if}
  </main>

  {#if searchOpen}
    <div class="search-popover">
      <Search size={15} />
      <input bind:value={searchQuery} placeholder="Search terminal output" aria-label="Search terminal output" />
      <span>Enter</span>
      <button onclick={() => { searchOpen = false; searchQuery = ""; }} aria-label="Close search"><X size={14} /></button>
    </div>
  {/if}

  {#if settingsOpen}<SettingsPanel {settings} onChange={changeSettings} onClose={() => settingsOpen = false} />{/if}
</div>
