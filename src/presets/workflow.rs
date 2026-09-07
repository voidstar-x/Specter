//! Workflow preset registry — JSON files under `workflow-presets/<domain>/`.
//!
//! Each file declares one system-shipped workflow (the in-app
//! equivalent of the upstream-Mike "built-in" entries that used to
//! live in `frontend/.../builtinWorkflows.ts`). Drop a JSON into the
//! right domain subfolder and restart the app — the registry will
//! merge it into `/workflow` responses with `is_system: true` and a
//! `null` user_id so the existing UI grey-out logic kicks in.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

/// Single column definition inside a tabular workflow's
/// `columns_config`. Mirrors `ColumnConfig` in the frontend.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowColumn {
    pub index: i64,
    pub name: String,
    /// Per-cell extraction prompt. Free-form Markdown.
    pub prompt: String,
    /// One of: `text` | `bulleted_list` | `number` | `currency` |
    /// `monetary_amount` | `percentage` | `yes_no` | `date` | `tag`.
    /// Optional — defaults to `text` at consumer side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// One workflow preset loaded from JSON. The shape projects directly
/// into the Specter workflow JSON the `/workflow` endpoint emits — the
/// only extra fields on the wire (`is_system`, `created_at`,
/// `user_id`) are synthesised by the route handler when it merges
/// presets into a list response, so they don't appear here.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowPreset {
    /// Stable identifier. Convention: `builtin-<slug>`. Used as the
    /// row id on the wire and by the frontend to detect built-ins.
    pub id: String,
    pub title: String,
    /// One of `assistant` | `tabular`.
    #[serde(rename = "type")]
    pub kind: String,
    /// Primary professional vertical — see `crate::domain::DOMAINS`.
    /// Required. Also the folder the preset lives under by convention.
    pub domain: String,
    /// Additional domains this preset should also surface under, beyond
    /// the primary `domain`. Mirrors the DOCX-template
    /// `also_applicable_to` mechanism: a workflow useful to more than
    /// one vertical (e.g. fixed-asset analysis is relevant to both
    /// `fiscale` and `finance`) is registered ONCE and listed in every
    /// applicable domain's picker. Empty / omitted = the preset shows
    /// strictly under its primary `domain`. Each entry is validated
    /// against the canonical domain set at load time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub also_applicable_to: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub practice: Option<String>,
    /// Free-form Markdown system prompt. Required for assistant
    /// workflows; for tabular workflows it sets the overall posture
    /// while each `columns_config` entry carries the per-cell prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_md: Option<String>,
    /// Tabular column schema. Optional (assistant workflows omit it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns_config: Option<Vec<WorkflowColumn>>,
    /// Optional DocxTemplate id this workflow's output should be
    /// formatted with. When set on an `assistant`-type workflow, the
    /// chat handler:
    ///   1. Injects the template's authoring prompt (auto-generated
    ///      from its sidecar fields) into the system message.
    ///   2. Encourages the LLM to finish the conversation by calling
    ///      `generate_docx(template_id=..., body_md=..., metadata=...)`.
    /// Omitted → workflow produces plain chat output; the user can
    /// still ask for a docx after the fact via the bare
    /// `generate_docx` tool. The wiring is opt-in per template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_output_template: Option<String>,
}

impl WorkflowPreset {
    /// True when this preset should appear for `target`: matches the
    /// primary `domain` or any entry in `also_applicable_to`. With
    /// `target = None` (no domain filter) it always matches. Mirrors
    /// `DocxTemplate::matches_domain`.
    pub fn matches_domain(&self, target: Option<&str>) -> bool {
        match target {
            None => true,
            Some(d) => self.domain == d || self.also_applicable_to.iter().any(|x| x == d),
        }
    }

    /// Render the preset as the JSON shape the `/workflow` endpoint
    /// serves to the frontend. Adds the synthesised fields (`is_system`,
    /// `user_id`, `created_at`, `is_owner`) so consumers see the same
    /// schema as user-created rows from the DB.
    pub fn to_api_json(&self) -> Value {
        let columns = self
            .columns_config
            .as_ref()
            .map(|cols| serde_json::to_value(cols).unwrap_or(serde_json::json!([])))
            .unwrap_or(serde_json::json!([]));
        serde_json::json!({
            "id": self.id,
            "user_id": null,
            "title": self.title,
            "type": self.kind,
            "prompt_md": self.prompt_md,
            "columns_config": columns,
            "practice": self.practice,
            "domain": self.domain,
            "also_applicable_to": self.also_applicable_to,
            "default_output_template": self.default_output_template,
            "created_at": "",
            "is_system": true,
            "is_owner": false,
        })
    }
}

/// Walk every JSON file in `dir` (one level of subdirectory recursion
/// for domain folders), parse each as a `WorkflowPreset`, validate
/// minimal invariants. Broken files are skipped with a `tracing::warn`
/// — one bad preset doesn't take down the rest.
pub fn load_workflow_presets(dir: &Path) -> Result<Vec<WorkflowPreset>> {
    let mut out: Vec<WorkflowPreset> = Vec::new();
    let files = match super::collect_json_files(dir) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!(
                "[workflow-presets] directory {} not found; no presets loaded",
                dir.display()
            );
            return Ok(out);
        }
        Err(e) => return Err(anyhow::anyhow!("read {}: {}", dir.display(), e)),
    };

    for path in files {
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    "[workflow-presets] skip {} (read error): {}",
                    path.display(),
                    e
                );
                continue;
            }
        };
        match serde_json::from_slice::<WorkflowPreset>(&bytes) {
            Ok(mut p) => {
                if p.id.is_empty() || p.title.is_empty() {
                    tracing::warn!(
                        "[workflow-presets] skip {} (id/title empty)",
                        path.display()
                    );
                    continue;
                }
                if p.kind != "assistant" && p.kind != "tabular" {
                    tracing::warn!(
                        "[workflow-presets] skip {} (type {} not in [assistant, tabular])",
                        path.display(),
                        p.kind
                    );
                    continue;
                }
                if !crate::domain::is_valid(&p.domain) {
                    tracing::warn!(
                        "[workflow-presets] skip {} (domain {} not in canonical set)",
                        path.display(),
                        p.domain
                    );
                    continue;
                }
                // Sanitise also_applicable_to: drop non-canonical
                // entries and any redundant listing of the primary
                // domain (warn, don't kill the preset). Keeps the
                // cross-domain surface honest without making one bad
                // entry take down an otherwise-valid workflow.
                {
                    let primary = p.domain.clone();
                    let before = p.also_applicable_to.len();
                    p.also_applicable_to.retain(|d| {
                        if d == &primary {
                            tracing::warn!(
                                "[workflow-presets] {}: also_applicable_to redundantly lists primary domain {} — dropping",
                                p.id, d
                            );
                            return false;
                        }
                        if !crate::domain::is_valid(d) {
                            tracing::warn!(
                                "[workflow-presets] {}: also_applicable_to entry {} not canonical — dropping",
                                p.id, d
                            );
                            return false;
                        }
                        true
                    });
                    let _ = before;
                }
                tracing::info!(
                    "[workflow-presets] loaded {} ({}, {}, domain={})",
                    p.id,
                    p.kind,
                    p.title,
                    p.domain
                );
                out.push(p);
            }
            Err(e) => {
                tracing::warn!(
                    "[workflow-presets] skip {} (parse error): {}",
                    path.display(),
                    e
                );
            }
        }
    }

    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity-check every JSON file shipped under `config/workflow-presets/`:
    /// the strongly-typed loader must accept all of them, every id must
    /// be unique, every `domain` must be in the canonical set and every
    /// preset's `kind` must be one of `assistant` | `tabular`. Catches a
    /// typo in a new preset before it disappears silently at startup.
    #[test]
    fn shipped_workflow_presets_all_load_cleanly() {
        let dir = crate::presets::config_subdir("workflow-presets");
        let presets = load_workflow_presets(&dir).expect("load shipped workflow presets");
        assert!(
            !presets.is_empty(),
            "no shipped workflow presets found under {}",
            dir.display()
        );

        let mut seen = std::collections::HashSet::new();
        for p in &presets {
            assert!(
                seen.insert(p.id.clone()),
                "duplicate workflow-preset id: {}",
                p.id
            );
            assert!(
                crate::domain::is_valid(&p.domain),
                "preset {} has non-canonical domain {}",
                p.id,
                p.domain
            );
            assert!(
                p.kind == "assistant" || p.kind == "tabular",
                "preset {} has unexpected kind {}",
                p.id,
                p.kind
            );
        }
    }

    #[test]
    fn matches_domain_primary_also_and_none() {
        let mut p = WorkflowPreset {
            id: "x".into(),
            title: "X".into(),
            kind: "tabular".into(),
            domain: "insurance".into(),
            also_applicable_to: vec!["finance".into()],
            practice: None,
            prompt_md: None,
            columns_config: None,
            default_output_template: None,
        };
        assert!(p.matches_domain(None), "no filter ⇒ always matches");
        assert!(p.matches_domain(Some("insurance")), "primary domain matches");
        assert!(p.matches_domain(Some("finance")), "also_applicable_to matches");
        assert!(!p.matches_domain(Some("legal")), "unrelated domain does not match");
        p.also_applicable_to.clear();
        assert!(!p.matches_domain(Some("finance")), "without also_applicable_to, secondary no longer matches");
    }

    #[test]
    fn shipped_legal_presets_have_expected_ids() {
        let dir = crate::presets::config_subdir("workflow-presets");
        let presets = load_workflow_presets(&dir).expect("load legal presets");
        let actual: std::collections::BTreeSet<&str> =
            presets.iter().map(|p| p.id.as_str()).collect();
        let expected: std::collections::BTreeSet<&str> = [
            "ai-act-conformity-eu",
            "ai-governance-assessment",
            "ai-governance-assessment-au",
            "ai-governance-assessment-sg",
            "cloud-saas-dpa-review",
            "consultation-response-au",
            "consultation-response-sg",
            "content-online-safety",
            "dpia-cross-border",
            "dpia-cross-border-au",
            "dpia-cross-border-sg",
            "gdpr-pdpa-compliance",
            "horizon-scan-obligation-map",
            "horizon-scan-obligation-map-au",
            "horizon-scan-obligation-map-sg",
            "nda-review-sg",
            "online-safety-code-au",
            "online-safety-code-sg",
            "oss-licence-review",
            "privacy-notice-consent-au",
            "privacy-notice-consent-sg",
            "saas-supply-review-sg",
            "statutory-analysis",
            "statutory-analysis-au",
            "statutory-analysis-sg",
        ].into_iter().collect();
        assert_eq!(presets.len(), 25);
        assert_eq!(actual, expected);
    }

    #[test]
    fn shipped_workflows_are_legal_tabular_with_columns() {
        let dir = crate::presets::config_subdir("workflow-presets");
        let presets = load_workflow_presets(&dir).expect("load legal presets");
        assert_eq!(presets.len(), 25);
        for p in presets {
            assert_eq!(p.domain, "legal", "{} domain", p.id);
            assert_eq!(p.kind, "tabular", "{} type", p.id);
            assert!(p.matches_domain(Some("legal")));
            let columns = p.columns_config.expect("tabular columns required");
            assert!(!columns.is_empty(), "{} has no columns", p.id);
            let mut indices = std::collections::HashSet::new();
            for column in columns {
                assert!(indices.insert(column.index), "{} duplicate column index", p.id);
                assert!(!column.name.trim().is_empty(), "{} empty column name", p.id);
                assert!(!column.prompt.trim().is_empty(), "{} empty column prompt", p.id);
            }
        }
    }

    #[test]
    fn shipped_workflows_do_not_reference_docx_templates() {
        let dir = crate::presets::config_subdir("workflow-presets");
        let presets = load_workflow_presets(&dir).expect("load legal presets");
        assert_eq!(presets.len(), 25);
        for p in presets {
            assert!(p.default_output_template.is_none(), "{} references a DOCX template", p.id);
        }
    }
}
