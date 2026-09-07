# Specter v0.5.6 — Chat composer UX fixes

Two bug fixes in the chat composer that surfaced during hands-on
testing of the v0.5.5 cycle, plus an opt-in experimental local-only
LLM mode for Ollama users (off by default).

## Bug fixes

### Doc picker — project scope

The "Sfoglia tutti" picker inside a project-scoped chat used to show
**every document the user had ever uploaded anywhere** — including
documents from other projects and standalone-chat attachments. It
now restricts to the current project's documents via the existing
`?project_id=…` filter in `documents.rs::list_documents`.

Standalone chats (Assistant tool, no project attached) keep the
global picker unchanged — that's the path the user takes to add a
library-wide doc to a fresh conversation.

### New-chat-in-project confirm modal

Clicking `+` (new chat) in the sidebar while a project-scoped chat
was active used to silently inherit the project on the new chat —
the user had no way to know the chip had carried over until they
glanced at the composer. Reported as confusing.

The new flow:

* If the active chat **is in a project**, clicking `+` now opens a
  confirm modal: *"Stai lavorando dentro un progetto. Vuoi
  mantenere il progetto associato alla nuova chat?"* with two
  action buttons — **Chat indipendente** / **Sì, mantieni il
  progetto** — and implicit cancel via the X / Esc / backdrop
  click.
* If the active chat **is not in a project**, the flow is unchanged
  (no modal, instant new chat).

Internally this also fixes the long-standing "chip persists
silently" bug where the composer's `$effect` early-returned on a
null `activeProjectId` instead of clearing the chip. A new monotonic
`chatStore.clearProjectTick` lets the modal's "Chat indipendente"
branch reset the chip without race conditions.

## New (opt-in, experimental)

### Local-only LLM mode for Ollama users

New toggle in Settings → Modelli LLM → "Modalità sicura locale".
**Off by default**, retro-compatible with existing local-provider
configurations.

When on:
* The local provider's base URL is locked to `http://localhost:11434`
  (loopback only — refuses LAN endpoints or public IPs).
* The chat composer's model picker collapses to two curated entries
  (Qwen 3.5 4B `q4_K_M` and Gemma 4 E2B IT GGUF `Q4_K_M`), both
  derived through Ollama Modelfiles Specter creates on demand with
  thinking suppression baked in.
* Install / cancel / parallel-download UX directly from Settings,
  with real-time progress streaming.

Backed by:
* New module [`src/llm/ollama_manager.rs`](src/llm/ollama_manager.rs)
  wrapping [`ollama-rs`](https://crates.io/crates/ollama-rs) 0.3.
* Schema migration **0032** — adds `user_settings.local_secure_mode
  INTEGER DEFAULT 0`.
* 13 new unit tests across `llm::local` and `llm::ollama_manager`
  (15/15 + 6/6 green).

Treat this as an **experimental preview** for the v0.6.x line. The
mechanism is feature-complete and tested, but the UX around model
discovery / context-window tuning / cross-platform Ollama detection
is still being refined. Power users who want to try it can flip the
toggle; everyone else can ignore it — the rest of the local
provider works exactly as before.

## Downloads

Pre-built MSIs for Windows:

- `Specter_0.5.6_x64.msi` — Windows x86_64
- `Specter_0.5.6_arm64.msi` — Windows ARM64, Snapdragon X Elite
  native

Drop-in replacement for v0.5.5.

## Migration notes

* New schema migration **0032** is applied automatically on first
  launch. The only column added defaults to 0 — existing installs
  preserve their custom Ollama URL and free-form model id.
* New direct dependencies: `ollama-rs = 0.3` (with `stream`) and
  `async-stream = 0.3`. Both pure Rust, no native libs added.

## License

Specter is distributed under **AGPL-3.0-only**. The Specter
wordmark and logo are trademarks; see `NOTICE.md`. The full
licence text is available in-app under **Settings → Licenza**.
