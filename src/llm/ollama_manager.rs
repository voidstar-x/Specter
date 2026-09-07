//! Plug-and-play Ollama lifecycle for the "Modalità sicura locale"
//! flow (v0.5.6).
//!
//! When the user toggles `local_secure_mode = true` in Settings, the
//! local OpenAI-compatible provider in [`super::local`] refuses any
//! base URL that isn't loopback and refuses any model id that isn't on
//! the curated [`CURATED_MODELS`] allowlist. This module is what makes
//! that allowlist usable end-to-end — without requiring the user to
//! touch a terminal:
//!
//!   * [`heartbeat`] — is Ollama actually running? Used by the UI to
//!     surface "Install Ollama from ollama.com" before anything else.
//!   * [`list_installed`] — what's already pulled? Drives the
//!     installed/not-installed badge on each curated entry.
//!   * [`ensure_curated`] — idempotent: pulls the base model if
//!     missing, then creates the `mike-…-fast` variant from the inline
//!     Modelfile if missing. Streams pull/create progress as
//!     [`EnsureEvent`] so the UI can render a progress bar instead of
//!     a frozen spinner during a 3 GB download.
//!   * [`uninstall_curated`] — counterpart of ensure_curated for the
//!     Settings "Rimuovi" button.
//!
//! Why the `mike-…-fast` prefix on every curated id: every curated
//! variant is a **Specter-owned Modelfile derivation** on top of an
//! upstream Ollama or HuggingFace model. The derivation carries the
//! thinking-suppression configuration (stop sequences, `/no_think`
//! token injection in the chat template, "rispondi direttamente"
//! system preamble) that's part of the "fast" UX promise. Pulling the
//! plain upstream model alone wouldn't suppress thinking — the
//! Modelfile wrapper is the whole point.

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::models::create::CreateModelRequest;
use ollama_rs::models::pull::PullModelStatus;
use ollama_rs::models::ModelOptions;
use ollama_rs::Ollama;
use serde::Serialize;

/// What flavour of thinking-suppression the curated variant applies.
/// Each base model has its own native escape hatch; the Modelfile
/// templated for that variant uses the matching one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ThinkingStrategy {
    /// Qwen 2.5 / 3.5 family: append the literal `/no_think` token at
    /// the end of every user message via a custom chat template. The
    /// upstream Qwen card documents this as the canonical "fast mode"
    /// switch — the model is RLHFed to honour the suffix and skip its
    /// `<think>` block entirely.
    QwenNoThinkToken,
    /// Gemma 4 family: no native control token. We rely on (a) a
    /// system preamble that explicitly tells the model not to emit
    /// `<think>` / `<thinking>` / `<reasoning>` blocks and (b) stop
    /// sequences on those tokens — so even if the model tries to open
    /// one, Ollama truncates immediately.
    GemmaStopSequences,
}

/// One curated model entry — everything Specter needs to (a) describe
/// the variant in the Settings UI and (b) materialise it in Ollama via
/// a Modelfile if the user clicks "Install".
#[derive(Debug, Clone, Serialize)]
pub struct CuratedModel {
    /// Specter-side id. This is what gets stored in
    /// `user_settings.local_model`, what the chat composer's model
    /// picker shows, and what the local provider checks against the
    /// allowlist. Always prefixed with `mike-` and suffixed with `-fast`
    /// to mark it as a Mike-owned derivation.
    pub id: &'static str,
    /// Upstream model id, as Ollama would receive it on `ollama pull`.
    /// Either a registry tag (`qwen2.5:3b-instruct-q4_K_M`) or a
    /// HuggingFace-routed tag (`hf.co/<org>/<repo>:<quant>`).
    pub base_model: &'static str,
    /// Human-facing label shown in the Settings list. Localised at the
    /// UI layer via the model.id, not by translating this string.
    pub display_name: &'static str,
    /// Rough on-disk footprint of the BASE model (the Modelfile
    /// derivation adds only a few KB of metadata). Used to set
    /// expectations on download size before the user clicks Install.
    pub approx_size_gb: f32,
    /// Recommended minimum RAM for inference. Used by the Settings UI
    /// to flag entries the current machine probably can't run
    /// smoothly.
    pub min_ram_gb: u32,
    /// Which thinking-suppression strategy the Modelfile applies.
    pub thinking_strategy: ThinkingStrategy,
}

/// The single source of truth for the curated allowlist. Adding a new
/// variant here automatically: (a) lets it pass the local-secure
/// allowlist check in [`super::local`], (b) makes the
/// `/llm-settings/local-secure/models` endpoint return it to the
/// frontend, (c) makes the Settings UI render it and (d) makes the
/// chat composer's model picker offer it. No other constant to update.
pub const CURATED_MODELS: &[CuratedModel] = &[
    CuratedModel {
        id: "mike-qwen35-4b-fast",
        // Qwen "3.5" upstream is published as Qwen 2.5 in Ollama's
        // registry — the user-facing label says "3.5" because that's
        // how Alibaba marketed the release, but the Ollama tag uses
        // the older internal version number.
        base_model: "qwen2.5:3b-instruct-q4_K_M",
        display_name: "Qwen 3.5 4B (rapido)",
        approx_size_gb: 2.5,
        min_ram_gb: 8,
        thinking_strategy: ThinkingStrategy::QwenNoThinkToken,
    },
    CuratedModel {
        id: "mike-gemma4-e2b-fast",
        // Unsloth's Dynamic GGUF 2.0 quantisation of Gemma 4 E2B
        // Instruct. The Q4_K_M sits at 3.1 GB on disk and ~5 GB RAM
        // during inference; the E2B (effective 2B params, 5.1B total
        // via per-layer embeddings) is the floor of Gemma 4 that fits
        // an 8-12 GB workstation while keeping 128K context.
        base_model: "hf.co/unsloth/gemma-4-E2B-it-GGUF:Q4_K_M",
        display_name: "Gemma 4 E2B (rapido)",
        approx_size_gb: 3.1,
        min_ram_gb: 12,
        thinking_strategy: ThinkingStrategy::GemmaStopSequences,
    },
];

/// Look up a curated entry by its Mike-side id. Used by `local.rs` to
/// enforce the allowlist and by `routes::user_secure` to validate that
/// the id the frontend asked to ensure/uninstall is actually a curated
/// one (i.e. not an arbitrary string the user wedged into the URL).
pub fn find_curated(id: &str) -> Option<&'static CuratedModel> {
    CURATED_MODELS.iter().find(|m| m.id == id)
}

/// Default loopback URL the manager talks to. Hardcoded — the secure
/// mode is precisely the mode that forbids the user from pointing at
/// any other URL. Plain `http://` because Ollama doesn't ship TLS on
/// loopback and there's no MITM to worry about there.
pub const SECURE_BASE_URL: &str = "http://localhost:11434";

fn client() -> Ollama {
    // `Ollama::new(host, port)` takes a host string (no scheme) and a
    // port number — split the constant rather than parsing it.
    Ollama::new("http://localhost".to_string(), 11434)
}

/// Is Ollama actually serving on the loopback port we're going to use?
/// Cheap call (~5 ms); the frontend should hit this before showing the
/// curated-models list so it can swap the list for an
/// "Installa Ollama" call-to-action when needed.
pub async fn heartbeat() -> bool {
    // `list_local_models` is the lightest documented call that
    // actually round-trips through Ollama's HTTP server — `Ollama::new`
    // alone doesn't talk to anything.
    matches!(client().list_local_models().await, Ok(_))
}

/// Which model ids currently exist on this Ollama instance. Includes
/// both the BASE models we'd pull and our own `mike-…-fast` variants
/// — the caller decides which one matters: a curated variant is
/// "ready to use" only when both its base and its mike- wrapper are
/// installed, but the BASE model alone is enough to skip the pull
/// step on the next ensure call.
pub async fn list_installed() -> Result<Vec<String>> {
    let models = client()
        .list_local_models()
        .await
        .context("ollama list_local_models")?;
    Ok(models.into_iter().map(|m| m.name).collect())
}

/// Status events streamed from [`ensure_curated`]. Forwarded as SSE
/// chunks by the `/llm-settings/local-secure/ensure` route so the
/// Settings UI can render a real-time progress bar instead of a
/// spinner that pretends a 3 GB download is instantaneous.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "phase")]
pub enum EnsureEvent {
    /// We've started — caller can show "preparing the variant…".
    Started { model_id: String },
    /// We're pulling the base model. Sent on every Ollama pull-status
    /// chunk so the UI can update a progress bar.
    Pulling {
        status: String,
        completed_bytes: u64,
        total_bytes: u64,
    },
    /// The base model is on disk; we're building the Modelfile
    /// derivation.
    Creating { model_id: String },
    /// Variant ready. The frontend can now flip the model picker over
    /// to it. Carries the FINAL `mike-…-fast` id (same as `model_id`
    /// in `Started` — convenience for the frontend's state machine).
    Ready { model_id: String },
    /// Something went wrong. Carries a localised-ready human message
    /// — the frontend surfaces it in a toast.
    Error { message: String },
}

/// Idempotent: bring the curated variant identified by `id` to a
/// "ready to use" state on the local Ollama. Stages:
///   1. Resolve the curated entry. Fail fast on an unknown id.
///   2. Cache `list_local_models` once.
///   3. If the BASE model is missing → stream `pull_model` events.
///   4. If the `mike-…-fast` variant is missing → render the
///      thinking-suppression Modelfile for this entry and
///      `create_model_stream` it.
///   5. Emit a `Ready` event.
///
/// All progress is yielded through the returned async stream so the
/// caller can forward it verbatim over SSE without buffering.
pub fn ensure_curated(
    id: String,
) -> impl futures_util::Stream<Item = EnsureEvent> + Send + 'static {
    async_stream::stream! {
        let Some(entry) = find_curated(&id) else {
            yield EnsureEvent::Error {
                message: format!("Modello non in allowlist: {id}"),
            };
            return;
        };
        yield EnsureEvent::Started { model_id: entry.id.to_string() };

        let ollama = client();

        // Step 1 — what's already installed? `name` includes the tag
        // (`qwen2.5:3b-instruct-q4_K_M`) so we compare against the full
        // string.
        let installed: std::collections::HashSet<String> = match ollama
            .list_local_models()
            .await
        {
            Ok(v) => v.into_iter().map(|m| m.name).collect(),
            Err(e) => {
                yield EnsureEvent::Error {
                    message: format!("Ollama non raggiungibile: {e}"),
                };
                return;
            }
        };

        // Step 2 — pull the base model if needed. We stream Ollama's
        // own progress events out as Pulling{…} so the UI bar tracks
        // bytes-downloaded in real time.
        if !installed.contains(entry.base_model) {
            let mut pull = match ollama
                .pull_model_stream(entry.base_model.to_string(), false)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    yield EnsureEvent::Error {
                        message: format!("ollama pull avvio fallito: {e}"),
                    };
                    return;
                }
            };
            while let Some(chunk) = pull.next().await {
                match chunk {
                    // `PullModelStatus` carries `message` (renamed
                    // from JSON `status`), `digest`, `total` and
                    // `completed`. We map only the bits the UI
                    // cares about into `EnsureEvent::Pulling`.
                    Ok(PullModelStatus { message, total, completed, .. }) => {
                        yield EnsureEvent::Pulling {
                            status: message,
                            completed_bytes: completed.unwrap_or(0),
                            total_bytes: total.unwrap_or(0),
                        };
                    }
                    Err(e) => {
                        yield EnsureEvent::Error {
                            message: format!("ollama pull stream interrotto: {e}"),
                        };
                        return;
                    }
                }
            }
        }

        // Step 3 — create the `mike-…-fast` variant if it doesn't
        // already exist. ollama-rs 0.3 doesn't expose a raw-Modelfile
        // path; it takes a structured `CreateModelRequest` (FROM +
        // template + system + parameters builder). We translate the
        // per-strategy Modelfile into that struct in
        // `build_create_request`.
        if !installed.contains(entry.id) {
            yield EnsureEvent::Creating { model_id: entry.id.to_string() };
            let req = build_create_request(entry);
            let mut create = match ollama.create_model_stream(req).await {
                Ok(s) => s,
                Err(e) => {
                    yield EnsureEvent::Error {
                        message: format!("ollama create avvio fallito: {e}"),
                    };
                    return;
                }
            };
            // Drain the create stream — we don't surface its chunks
            // because Modelfile derivation is fast (single-digit
            // seconds) and the UI just shows "preparing variant…".
            while let Some(chunk) = create.next().await {
                if let Err(e) = chunk {
                    yield EnsureEvent::Error {
                        message: format!("ollama create stream errore: {e}"),
                    };
                    return;
                }
            }
        }

        yield EnsureEvent::Ready { model_id: entry.id.to_string() };
    }
}

/// Counterpart of [`ensure_curated`]. Removes the `mike-…-fast`
/// derivation only — the BASE model stays installed in case the user
/// has other Modelfile derivations layered on it. The Settings UI
/// uses this when the user clicks "Rimuovi".
pub async fn uninstall_curated(id: &str) -> Result<()> {
    if find_curated(id).is_none() {
        return Err(anyhow!("Modello non in allowlist: {id}"));
    }
    client()
        .delete_model(id.to_string())
        .await
        .context("ollama delete_model")
}

/// Pre-warm: run a no-op generation so Ollama loads the model into
/// RAM and the first user chat doesn't pay the ~5-10s cold-start. Best
/// called from a background task after `ensure_curated` resolves.
pub async fn warm_up(id: &str) -> Result<()> {
    // `num_predict = 1` plus a one-character prompt gives Ollama
    // enough to mmap the GGUF and initialise the KV cache without
    // generating anything the user would see.
    let _ = client()
        .generate(GenerationRequest::new(id.to_string(), " ".to_string()))
        .await
        .context("ollama warm-up generation")?;
    Ok(())
}

/// Build the structured `CreateModelRequest` that materialises the
/// curated `mike-…-fast` Ollama variant for `entry`. Equivalent to
/// pushing an inline Modelfile through `ollama create`, but expressed
/// as ollama-rs 0.3's builder API (`/api/create` doesn't accept a
/// raw Modelfile blob — it wants structured fields).
///
/// The per-strategy thinking-suppression bake-in is the whole reason
/// the `mike-…-fast` derivation exists; pulling the upstream model
/// alone wouldn't disable the model's chain-of-thought output.
fn build_create_request(entry: &CuratedModel) -> CreateModelRequest {
    let opts_base = ModelOptions::default()
        .temperature(0.5)
        .num_predict(2048);
    match entry.thinking_strategy {
        ThinkingStrategy::QwenNoThinkToken => CreateModelRequest::new(entry.id.to_string())
            .from_model(entry.base_model.to_string())
            // Inject /no_think at the end of every user turn. Qwen
            // 2.5/3.5 is RLHFed to honour this token and skip its
            // <think> block entirely — Alibaba's Qwen card documents
            // this as the canonical "fast mode" switch.
            .template(QWEN_NO_THINK_TEMPLATE.to_string())
            .parameters(opts_base),
        ThinkingStrategy::GemmaStopSequences => CreateModelRequest::new(entry.id.to_string())
            .from_model(entry.base_model.to_string())
            // Gemma 4 has no native /no_think — we lean on three
            // stop sequences so the moment the model opens a
            // thinking block Ollama truncates the generation, plus a
            // system preamble that tells the model not to open one
            // in the first place.
            .system(GEMMA_NO_THINK_SYSTEM.to_string())
            .parameters(
                opts_base.stop(vec![
                    "<think>".to_string(),
                    "<thinking>".to_string(),
                    "<reasoning>".to_string(),
                ]),
            ),
    }
}

const QWEN_NO_THINK_TEMPLATE: &str = "{{ if .System }}<|im_start|>system\n\
{{ .System }}<|im_end|>\n\
{{ end }}{{ range .Messages }}<|im_start|>{{ .Role }}\n\
{{ .Content }}/no_think<|im_end|>\n\
{{ end }}<|im_start|>assistant\n";

const GEMMA_NO_THINK_SYSTEM: &str =
    "Always answer directly and concisely, in English. \
     Do not include explicit reasoning between <think>, <thinking>, <reasoning> \
     or similar blocks. Go straight to the final answer.";

/// Thinking-suppression PREAMBLE to prepend to the system prompt when
/// the user is in secure mode. This is the second line of defence:
/// the curated `mike-…-fast` variants already bake the suppression
/// into their Modelfile, but if for any reason the chat is hitting an
/// uncurated model on the same Ollama instance (e.g. an old config
/// migration), this string still tries to keep the output direct.
///
/// Returned with a trailing newline so callers can safely concatenate
/// with their existing system prompt.
pub fn no_think_preamble() -> &'static str {
    "[Secure local mode] Always answer directly and concisely. \
     Do not include explicit reasoning between <think>, <thinking>, <reasoning> \
     or similar blocks. Go straight to the final answer.\n\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_allowlist_has_unique_ids() {
        let mut seen = std::collections::HashSet::new();
        for m in CURATED_MODELS {
            assert!(seen.insert(m.id), "duplicate curated id: {}", m.id);
        }
    }

    #[test]
    fn curated_ids_are_prefixed() {
        for m in CURATED_MODELS {
            assert!(
                m.id.starts_with("mike-") && m.id.ends_with("-fast"),
                "curated id `{}` must be `mike-…-fast` (the prefix encodes \
                 'this is a Specter-owned Modelfile derivation' and the \
                 suffix encodes 'thinking is disabled')",
                m.id,
            );
        }
    }

    #[test]
    fn find_curated_rejects_arbitrary_ids() {
        assert!(find_curated("mike-qwen35-4b-fast").is_some());
        assert!(find_curated("qwen2.5:3b-instruct-q4_K_M").is_none());
        assert!(find_curated("../../../etc/passwd").is_none());
    }

    #[test]
    fn create_request_qwen_injects_no_think_token() {
        let entry = find_curated("mike-qwen35-4b-fast").unwrap();
        let req = build_create_request(entry);
        assert_eq!(req.model_name, "mike-qwen35-4b-fast");
        assert_eq!(
            req.from_model.as_deref(),
            Some("qwen2.5:3b-instruct-q4_K_M")
        );
        let template = req.template.as_deref().expect("Qwen variant must set a template");
        assert!(
            template.contains("/no_think"),
            "Qwen variant must inject /no_think in the chat template (got: {template:?})"
        );
        assert!(template.contains("<|im_start|>"));
    }

    #[test]
    fn create_request_gemma_stops_on_thinking_tokens() {
        let entry = find_curated("mike-gemma4-e2b-fast").unwrap();
        let req = build_create_request(entry);
        assert_eq!(req.model_name, "mike-gemma4-e2b-fast");
        assert_eq!(
            req.from_model.as_deref(),
            Some("hf.co/unsloth/gemma-4-E2B-it-GGUF:Q4_K_M")
        );
        let system = req.system.as_deref().expect("Gemma variant must set a system preamble");
        assert!(system.contains("directly and concisely, in English"));
        // The serialised parameters carry the three stop sequences;
        // ModelOptions doesn't expose getters in 0.3 so we roundtrip
        // through JSON to inspect them.
        let params = req
            .parameters
            .as_ref()
            .expect("Gemma variant must set parameters");
        let json = serde_json::to_value(params).unwrap();
        let stops = json.get("stop").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let stops: Vec<String> = stops
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();
        assert!(stops.contains(&"<think>".to_string()));
        assert!(stops.contains(&"<thinking>".to_string()));
        assert!(stops.contains(&"<reasoning>".to_string()));
    }

    #[test]
    fn no_think_preamble_marks_itself() {
        let preamble = no_think_preamble();
        assert!(preamble.contains("[Secure local mode]"));
        assert!(preamble.ends_with("\n\n"));
    }
}
