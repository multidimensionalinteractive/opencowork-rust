import { createSignal, createEffect, onMount, For, Show } from "solid-js";
import { createStore } from "solid-js/store";
import "./styles/design-system.css";

// ─── Types ───
interface Message {
	id: string;
	role: "user" | "assistant" | "system";
	content: string;
	timestamp: number;
	toolCalls?: ToolCall[];
}

interface ToolCall {
	id: string;
	name: string;
	status: "running" | "completed" | "error";
	output?: string;
}

interface Session {
	id: string;
	title: string;
	lastMessage: string;
	timestamp: number;
	pinned: boolean;
	messageCount: number;
}

// ─── Stores ───
const [sessions, setSessions] = createStore<Session[]>([
	{ id: "1", title: "OpenCoWork Architecture", lastMessage: "Let me analyze the crate structure...", timestamp: Date.now() - 3600000, pinned: true, messageCount: 24 },
	{ id: "2", title: "Rust Performance Tuning", lastMessage: "The Axum server benchmarks show...", timestamp: Date.now() - 7200000, pinned: false, messageCount: 18 },
	{ id: "3", title: "Telegram Bot Integration", lastMessage: "Teloxide adapter is ready", timestamp: Date.now() - 86400000, pinned: false, messageCount: 12 },
]);

const [messages, setMessages] = createStore<Message[]>([
	{ id: "m1", role: "user", content: "Can you analyze the performance difference between our Rust Axum server and the TypeScript Bun server?", timestamp: Date.now() - 120000 },
	{ id: "m2", role: "assistant", content: "Here's the breakdown:\n\n| Metric | Bun (TS) | Axum (Rust) | Delta |\n|--------|----------|-------------|-------|\n| Startup | ~800ms | ~12ms | **67x** |\n| Idle RAM | 85MB | 8MB | **10x** |\n| Routing | 15ms | 0.3ms | **50x** |\n| Binary | 180MB | 12MB | **15x** |\n\nThe Axum server uses `tokio::sync::broadcast` for SSE fan-out and `dashmap::DashMap` for lock-free concurrent state — both eliminate the GC pauses inherent in Bun's V8 runtime.", timestamp: Date.now() - 90000, toolCalls: [{ id: "tc1", name: "cargo_bench", status: "completed", output: "12.3ms startup, 0.31ms routing" }] },
	{ id: "m3", role: "user", content: "What about the Mojo acceleration for inference?", timestamp: Date.now() - 60000 },
]);

const [activeSession, setActiveSession] = createSignal("1");
const [showCmdPalette, setShowCmdPalette] = createSignal(false);
const [showShortcuts, setShowShortcuts] = createSignal(false);
const [cmdQuery, setCmdQuery] = createSignal("");
const [selectedCmdIdx, setSelectedCmdIdx] = createSignal(0);
const [inputValue, setInputValue] = createSignal("");
const [isTyping, setIsTyping] = createSignal(false);
const [sidebarCollapsed, setSidebarCollapsed] = createSignal(false);
const [toasts, setToasts] = createStore<{id: string; message: string; type: "success" | "error" | "info"}[]>([]);

// ─── Keyboard Shortcuts ───
onMount(() => {
	const handler = (e: KeyboardEvent) => {
		// Cmd+K / Ctrl+K → Command palette
		if ((e.metaKey || e.ctrlKey) && e.key === "k") {
			e.preventDefault();
			setShowCmdPalette(p => !p);
		}
		// Cmd+/ → Shortcuts
		if ((e.metaKey || e.ctrlKey) && e.key === "/") {
			e.preventDefault();
			setShowShortcuts(p => !p);
		}
		// Cmd+B → Toggle sidebar
		if ((e.metaKey || e.ctrlKey) && e.key === "b") {
			e.preventDefault();
			setSidebarCollapsed(p => !p);
		}
		// Escape → Close overlays
		if (e.key === "Escape") {
			setShowCmdPalette(false);
			setShowShortcuts(false);
		}
		// Cmd+N → New session
		if ((e.metaKey || e.ctrlKey) && e.key === "n") {
			e.preventDefault();
			createNewSession();
		}
	};
	window.addEventListener("keydown", handler);
	return () => window.removeEventListener("keydown", handler);
});

// ─── Commands ───
const commands = [
	{ icon: "➕", title: "New Session", desc: "Start a fresh conversation", shortcut: ["⌘", "N"], action: () => createNewSession() },
	{ icon: "🔍", title: "Search Sessions", desc: "Find past conversations", shortcut: ["⌘", "F"], action: () => {} },
	{ icon: "📁", title: "Open Workspace", desc: "Load a project directory", shortcut: ["⌘", "O"], action: () => {} },
	{ icon: "🎨", title: "Change Theme", desc: "Switch visual theme", shortcut: [], action: () => {} },
	{ icon: "📊", title: "View Benchmarks", desc: "Performance metrics dashboard", shortcut: [], action: () => {} },
	{ icon: "🔌", title: "Connect Remote", desc: "Add a remote worker", shortcut: [], action: () => {} },
	{ icon: "⚙️", title: "Settings", desc: "Configure OpenCoWork", shortcut: ["⌘", ","], action: () => {} },
	{ icon: "❓", title: "Keyboard Shortcuts", desc: "View all shortcuts", shortcut: ["⌘", "/"], action: () => setShowShortcuts(true) },
];

function createNewSession() {
	const id = `s${Date.now()}`;
	setSessions(sessions.length, { id, title: "New Session", lastMessage: "", timestamp: Date.now(), pinned: false, messageCount: 0 });
	setActiveSession(id);
	setMessages([]);
}

function sendMessage() {
	const content = inputValue().trim();
	if (!content) return;

	const userMsg: Message = { id: `m${Date.now()}`, role: "user", content, timestamp: Date.now() };
	setMessages(messages.length, userMsg);
	setInputValue("");
	setIsTyping(true);

	// Simulate assistant response
	setTimeout(() => {
		setIsTyping(false);
		const aiMsg: Message = {
			id: `m${Date.now() + 1}`,
			role: "assistant",
			content: "Processing your request...",
			timestamp: Date.now(),
		};
		setMessages(messages.length, aiMsg);
	}, 1500);
}

function addToast(message: string, type: "success" | "error" | "info" = "info") {
	const id = `t${Date.now()}`;
	setToasts(toasts.length, { id, message, type });
	setTimeout(() => setToasts(t => t.filter(t_ => t_.id !== id)), 3000);
}

function filteredCommands() {
	const q = cmdQuery().toLowerCase();
	if (!q) return commands;
	return commands.filter(c => c.title.toLowerCase().includes(q) || c.desc.toLowerCase().includes(q));
}

function formatTime(ts: number) {
	const diff = Date.now() - ts;
	if (diff < 60000) return "now";
	if (diff < 3600000) return `${Math.floor(diff / 60000)}m`;
	if (diff < 86400000) return `${Math.floor(diff / 3600000)}h`;
	return `${Math.floor(diff / 86400000)}d`;
}

// ─── Components ───
function Sidebar() {
	return (
		<aside class={`sidebar ${sidebarCollapsed() ? "collapsed" : ""}`} style={sidebarCollapsed() ? { width: "64px" } : { width: "280px" }}>
			<div class="sidebar-header">
				<div class="sidebar-logo">
					<div class="logo-icon">🦀</div>
					<Show when={!sidebarCollapsed()}>
						<span>OpenCoWork</span>
					</Show>
				</div>
			</div>

			<Show when={!sidebarCollapsed()}>
				<div class="cmd-trigger" onClick={() => setShowCmdPalette(true)}>
					<span>🔍</span>
					<span>Search or command...</span>
					<kbd>⌘K</kbd>
				</div>

				<div class="session-list">
					<div class="session-group-label">Pinned</div>
					<For each={sessions.filter(s => s.pinned)}>
						{(s) => (
							<div class={`session-item ${activeSession() === s.id ? "active" : ""}`} onClick={() => setActiveSession(s.id)}>
								<span class="session-title">{s.title}</span>
								<span class="session-time">{formatTime(s.timestamp)}</span>
							</div>
						)}
					</For>

					<div class="session-group-label">Recent</div>
					<For each={sessions.filter(s => !s.pinned).sort((a, b) => b.timestamp - a.timestamp)}>
						{(s) => (
							<div class={`session-item ${activeSession() === s.id ? "active" : ""}`} onClick={() => setActiveSession(s.id)}>
								<span class="session-title">{s.title}</span>
								<span class="session-time">{formatTime(s.timestamp)}</span>
								<span class="session-pin" title="Pin">📌</span>
							</div>
						)}
					</For>
				</div>
			</Show>
		</aside>
	);
}

function TopBar() {
	const session = sessions.find(s => s.id === activeSession());
	return (
		<div class="top-bar">
			<div class="top-bar-title">
				<span>{session?.title ?? "OpenCoWork"}</span>
				<span style={{ "font-size": "11px", color: "var(--text-tertiary)" }}>
					{session?.messageCount ?? 0} messages
				</span>
			</div>
			<div class="top-bar-actions">
				<button class="toolbar-btn" title="Pin session">📌</button>
				<button class="toolbar-btn" title="Export">📤</button>
				<button class="toolbar-btn" title="Settings">⚙️</button>
			</div>
		</div>
	);
}

function ChatMessage(msg: Message) {
	return (
		<div class="message">
			<div class={`message-avatar ${msg.role}`}>
				{msg.role === "user" ? "M" : "OW"}
			</div>
			<div class="message-body">
				<div class="message-header">
					<span class="message-name">{msg.role === "user" ? "You" : "OpenCoWork"}</span>
					<span class="message-time">{formatTime(msg.timestamp)}</span>
				</div>
				<div class="message-content">
					{msg.content}
				</div>
				<Show when={msg.toolCalls}>
					<For each={msg.toolCalls}>
						{(tc) => (
							<div class={`tool-call ${tc.status}`}>
								<Show when={tc.status === "running"}>
									<div class="tool-spinner" />
								</Show>
								<span>🔧 {tc.name}</span>
								<Show when={tc.output}>
									<span style={{ "margin-left": "auto", "font-size": "11px" }}>{tc.output}</span>
								</Show>
							</div>
						)}
					</For>
				</Show>
				<div class="message-actions">
					<button class="message-action-btn">📋 Copy</button>
					<button class="message-action-btn">🔄 Regenerate</button>
					<button class="message-action-btn">👍</button>
					<button class="message-action-btn">👎</button>
				</div>
			</div>
		</div>
	);
}

function InputArea() {
	let textareaRef: HTMLTextAreaElement;

	const handleKeyDown = (e: KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			sendMessage();
		}
	};

	const autoResize = () => {
		if (textareaRef) {
			textareaRef.style.height = "auto";
			textareaRef.style.height = `${Math.min(textareaRef.scrollHeight, 200)}px`;
		}
	};

	return (
		<div class="input-area">
			<div class="input-container">
				<textarea
					ref={textareaRef!}
					class="input-textarea"
					placeholder="Ask anything... (Enter to send, Shift+Enter for new line)"
					value={inputValue()}
					onInput={(e) => { setInputValue(e.currentTarget.value); autoResize(); }}
					onKeyDown={handleKeyDown}
					rows={1}
				/>
				<div class="input-toolbar">
					<div class="input-toolbar-left">
						<button class="toolbar-btn" title="Attach file">📎</button>
						<button class="toolbar-btn" title="Code mode">💻</button>
						<button class="toolbar-btn" title="Web search">🌐</button>
					</div>
					<div class="input-toolbar-right">
						<span style={{ "font-size": "11px", color: "var(--text-tertiary)" }}>
							{inputValue().length > 0 ? `${inputValue().length} chars` : ""}
						</span>
						<button class="send-btn" onClick={sendMessage} disabled={!inputValue().trim()}>
							Send ↵
						</button>
					</div>
				</div>
			</div>
		</div>
	);
}

function CommandPalette() {
	return (
		<Show when={showCmdPalette()}>
			<div class="cmd-overlay" onClick={() => setShowCmdPalette(false)}>
				<div class="cmd-palette" onClick={(e) => e.stopPropagation()}>
					<input
						class="cmd-input"
						placeholder="Type a command or search..."
						value={cmdQuery()}
						onInput={(e) => { setCmdQuery(e.currentTarget.value); setSelectedCmdIdx(0); }}
						autofocus
					/>
					<div class="cmd-results">
						<div class="cmd-group-label">Actions</div>
						<For each={filteredCommands()}>
							{(cmd, i) => (
								<div
									class={`cmd-item ${selectedCmdIdx() === i() ? "selected" : ""}`}
									onClick={() => { cmd.action(); setShowCmdPalette(false); }}
									onMouseEnter={() => setSelectedCmdIdx(i())}
								>
									<div class="cmd-item-icon">{cmd.icon}</div>
									<div class="cmd-item-text">
										<div class="cmd-item-title">{cmd.title}</div>
										<div class="cmd-item-desc">{cmd.desc}</div>
									</div>
									<Show when={cmd.shortcut.length > 0}>
										<div class="cmd-item-shortcut">
											<For each={cmd.shortcut}>{(k) => <kbd>{k}</kbd>}</For>
										</div>
									</Show>
								</div>
							)}
						</For>
					</div>
				</div>
			</div>
		</Show>
	);
}

function ShortcutsOverlay() {
	const shortcuts = [
		{ label: "Command Palette", keys: ["⌘", "K"] },
		{ label: "New Session", keys: ["⌘", "N"] },
		{ label: "Toggle Sidebar", keys: ["⌘", "B"] },
		{ label: "Keyboard Shortcuts", keys: ["⌘", "/"] },
		{ label: "Send Message", keys: ["Enter"] },
		{ label: "New Line", keys: ["Shift", "Enter"] },
		{ label: "Close Overlay", keys: ["Esc"] },
	];

	return (
		<Show when={showShortcuts()}>
			<div class="shortcuts-overlay" onClick={() => setShowShortcuts(false)}>
				<div class="shortcuts-panel" onClick={(e) => e.stopPropagation()}>
					<div class="shortcuts-title">⌨️ Keyboard Shortcuts</div>
					<For each={shortcuts}>
						{(s) => (
							<div class="shortcut-row">
								<span class="shortcut-label">{s.label}</span>
								<div class="shortcut-keys">
									<For each={s.keys}>{(k) => <kbd>{k}</kbd>}</For>
								</div>
							</div>
						)}
					</For>
				</div>
			</div>
		</Show>
	);
}

function StatusBar() {
	return (
		<div class="status-bar">
			<div style={{ display: "flex", gap: "var(--space-md)" }}>
				<div class="status-item">
					<div class="status-dot" />
					<span>Connected</span>
				</div>
				<div class="status-item">
					<span>🦀 Axum Server</span>
				</div>
				<div class="status-item">
					<span>⚡ 12ms latency</span>
				</div>
			</div>
			<div style={{ display: "flex", gap: "var(--space-md)" }}>
				<div class="status-item">
					<span>v0.1.0</span>
				</div>
				<div class="status-item">
					<span>⌘K Commands</span>
				</div>
			</div>
		</div>
	);
}

function ToastContainer() {
	return (
		<div class="toast-container">
			<For each={toasts}>
				{(toast) => (
					<div class={`toast ${toast.type}`}>
						<span>{toast.type === "success" ? "✅" : toast.type === "error" ? "❌" : "ℹ️"}</span>
						<span>{toast.message}</span>
					</div>
				)}
			</For>
		</div>
	);
}

// ─── Main App ───
export default function App() {
	return (
		<div class={`app-shell ${sidebarCollapsed() ? "sidebar-collapsed" : ""}`}>
			<Sidebar />
			<div class="main-content">
				<TopBar />
				<div class="chat-area">
					<For each={messages}>
						{(msg) => <ChatMessage {...msg} />}
					</For>
					<Show when={isTyping()}>
						<div class="message">
							<div class="message-avatar assistant">OW</div>
							<div class="message-body">
								<div class="typing-indicator">
									<div class="typing-dot" />
									<div class="typing-dot" />
									<div class="typing-dot" />
								</div>
							</div>
						</div>
					</Show>
				</div>
				<InputArea />
				<StatusBar />
			</div>

			<CommandPalette />
			<ShortcutsOverlay />
			<ToastContainer />
		</div>
	);
}
