//! DOCX template registry — sidecar JSON files under
//! `config/docx-templates/<domain>/<slug>.json`. Layout is rendered directly
//! from the sidecar; no companion Word template is needed.
//!
//! Templates are optional, user-authored closing formatters. The LLM
//! produces Markdown; the renderer applies the sidecar layout and binds
//! `[PLACEHOLDERS]` to emit a print-ready document. No templates ship by default.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────
// Layout primitives
// ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Paper {
    pub size: String,
    #[serde(default = "default_orientation")]
    pub orientation: String,
    /// `"standard"` for A4 ordinary, `"uso_bollo"` for notarial deeds.
    /// When `uso_bollo`, the sibling `uso_bollo` block becomes required.
    #[serde(default = "default_paper_format")]
    pub format: String,
}

fn default_orientation() -> String {
    "portrait".to_string()
}

fn default_paper_format() -> String {
    "standard".to_string()
}

/// Special paper rules for notarial "uso bollo" deeds. Only present
/// when `paper.format == "uso_bollo"`. Captures the constraints listed
/// by the template: lines per page, mirror margins,
/// no blank lines allowed, marginal signature on every page except
/// the last.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsoBollo {
    /// Line spacing in typographic points (not `1.5` multiplier).
    /// Standard is `28.35` for the canonical 25-lines/facciata layout.
    pub line_spacing_pt_exact: f32,
    pub lines_per_facciata: u32,
    pub facciate_per_foglio: u32,
    #[serde(default)]
    pub mirror_margins: bool,
    #[serde(default)]
    pub duplex: bool,
    #[serde(default)]
    pub forbid_empty_lines: bool,
    #[serde(default)]
    pub marginal_signature_required: bool,
    #[serde(default)]
    pub signature_exclude_last_page: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarginsCm {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Typography {
    pub body_font: String,
    pub body_size_pt: f32,
    pub line_spacing: f32,
    #[serde(default)]
    pub paragraph_after_pt: f32,
    /// `"justify"` (legal/forense) or `"left"` (PA blocco americano).
    #[serde(default = "default_alignment")]
    pub alignment: String,
    #[serde(default)]
    pub first_line_indent_cm: f32,
}

fn default_alignment() -> String {
    "justify".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Footnotes {
    pub font: String,
    pub size_pt: f32,
    pub line_spacing: f32,
}

// ─────────────────────────────────────────────────────────────────────
// Authoring contract
// ─────────────────────────────────────────────────────────────────────

/// One step in the document's expected structure. The `id` is the
/// canonical English snake_case identifier (memory: English IDs,
/// localised display). The `title` is the heading text rendered into
/// the Word document — typically Italian for `it/` templates, but the
/// field carries whatever the template author wrote.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SectionSkeletonEntry {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Literal text to render in place of a heading — e.g. `"* * *"`
    /// for the inter-block separator used in atto difensivo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub render: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<String>,
    /// When `true`, this section is a *repeating block* (L3 automation) — e.g. one quesito → one risposta in CTU, one
    /// process card → one row in ISO. Renderer expects the LLM to
    /// produce a list under this section in the Markdown.
    #[serde(default)]
    pub repeating: bool,
}

/// Character-count vincoli, used by atto difensivo (D.M. 110/2023).
/// Map of `atto_type` → max chars. Renderer warns when body exceeds.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CharacterLimits {
    /// Map preserved verbatim so future `atto_type` values can be
    /// added without recompiling.
    #[serde(flatten)]
    pub by_atto_type: std::collections::HashMap<String, u64>,
}

/// Few-shot example pointing at a Markdown file in the same template
/// directory. Loaded lazily when the LLM asks for examples via
/// `describe_docx_template`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FewShotExample {
    pub label: String,
    /// Path relative to the template's sidecar JSON.
    pub path: String,
}

// ─────────────────────────────────────────────────────────────────────
// DocxTemplate root
// ─────────────────────────────────────────────────────────────────────

/// One template as parsed from disk. Every field except the bare
/// minimum (`id`, `display_name`, `paper`, `margins_cm`, `typography`)
/// is optional so a new template can be drafted incrementally.
///
/// Serialisation back to JSON via `to_api_json()` adds the synthesised
/// fields (`is_system: true`) that the route returns to the frontend.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocxTemplate {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    pub id: String,
    /// Map of locale code → display name. Renderer picks the entry
    /// matching the user's UI locale; falls back to `en` then to the
    /// first available entry.
    pub display_name: std::collections::HashMap<String, String>,

    pub category: String,
    /// Canonical primary domain — see `crate::domain::DOMAINS`.
    /// Templates are tagged with the domain where they were natively
    /// designed (legal letter → `legal`; ISO procedure → `compliance`).
    pub domain: String,
    /// Additional domains where the template is also useful. Allows
    /// a template like Parcella (primary `finance`) to surface for
    /// users with `default_domain = legal`, since every liberal
    /// professional issues parcelle. The filter on `GET /docx-templates?
    /// domain=X` returns a template when `X == domain` OR `X in
    /// also_applicable_to`. Empty / omitted means strictly domain-
    /// specific (e.g. ISO Procedure, Contratto di locazione).
    #[serde(default)]
    pub also_applicable_to: Vec<String>,
    pub locale: String,

    #[serde(default = "default_automation_level")]
    pub automation_level: String,
    #[serde(default = "default_placeholder_syntax")]
    pub placeholder_syntax: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<String>,

    // ── layout ──────────────────────────────────────────────────────
    pub paper: Paper,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uso_bollo: Option<UsoBollo>,
    pub margins_cm: MarginsCm,
    pub typography: Typography,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footnotes: Option<Footnotes>,

    /// Universal baseline of four document styles.
    /// Keys are canonical English IDs; values are the Word style names
    /// embedded in the generated document (localised per template).
    #[serde(default = "default_style_map_baseline")]
    pub style_map_baseline: std::collections::BTreeMap<String, String>,

    /// Template-specific style overrides on top of the baseline.
    #[serde(default)]
    pub style_map: std::collections::BTreeMap<String, String>,

    #[serde(default)]
    pub directives_supported: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_block: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer_block: Option<String>,
    /// `"manual"` (the LLM writes `1.`, `2.` in the heading text) or
    /// `"auto"` (the template uses Word numbering definitions).
    #[serde(default = "default_section_numbering")]
    pub section_numbering: String,

    // ── authoring contract ──────────────────────────────────────────
    #[serde(default)]
    pub section_skeleton: Vec<SectionSkeletonEntry>,

    /// Per-field micro-prompts for the LLM, telling it how to extract
    /// each required_metadata field from a chat conversation.
    #[serde(default)]
    pub field_prompts: std::collections::BTreeMap<String, String>,

    /// Names of metadata fields the LLM must collect before the
    /// renderer runs. Universal fields (LUOGO, DATA, MITTENTE, …) are
    /// always implicitly required and don't need to be listed here.
    #[serde(default)]
    pub required_metadata: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_limits: Option<CharacterLimits>,

    #[serde(default)]
    pub few_shot_examples: Vec<FewShotExample>,

    /// Optional author override appended to the auto-generated
    /// `prompt_md`. Use for jurisdiction-specific tone notes that
    /// don't fit cleanly in `field_prompts`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_md_extra: Option<String>,
}

fn default_schema_version() -> u32 {
    1
}

fn default_automation_level() -> String {
    "L1".to_string()
}

fn default_placeholder_syntax() -> String {
    "square_brackets".to_string()
}

fn default_section_numbering() -> String {
    "manual".to_string()
}

/// The 4 canonical paragraph styles every template inherits. Keys are
/// the IDs the renderer references; values are what the companion
/// `.dotx` actually defines (will be `"Corpo testo"` etc. for IT
/// templates, `"Body Text"` for future EN ones).
fn default_style_map_baseline() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    m.insert("body_text".to_string(), "Corpo testo".to_string());
    m.insert("section_heading".to_string(), "Titolo sezione".to_string());
    m.insert("citation".to_string(), "Citazione".to_string());
    m.insert("footnote".to_string(), "Note piè pagina".to_string());
    m
}

impl DocxTemplate {
    /// True if this template should surface for users whose active
    /// domain filter is `target`. Matches both the primary `domain`
    /// and any entry in `also_applicable_to`. With `target = None`
    /// every template matches — the no-filter case.
    pub fn matches_domain(&self, target: Option<&str>) -> bool {
        match target {
            None => true,
            Some(d) => self.domain == d || self.also_applicable_to.iter().any(|x| x == d),
        }
    }

    /// Resolve a display name for the given locale, with English
    /// fallback and last-resort first-entry pickup.
    pub fn display_name_for(&self, locale: &str) -> String {
        if let Some(name) = self.display_name.get(locale) {
            return name.clone();
        }
        if let Some(name) = self.display_name.get("en") {
            return name.clone();
        }
        self.display_name
            .values()
            .next()
            .cloned()
            .unwrap_or_else(|| self.id.clone())
    }

    /// Render the template as the JSON shape the `/docx-templates`
    /// endpoint serves. Adds synthesised fields (`is_system: true`,
    /// `is_owner: false`) so consumers see the same schema as future
    /// user-created rows from the DB.
    pub fn to_api_json(&self) -> Value {
        let mut v = serde_json::to_value(self).unwrap_or(serde_json::json!({}));
        if let Value::Object(ref mut map) = v {
            map.insert("is_system".to_string(), Value::Bool(true));
            map.insert("is_owner".to_string(), Value::Bool(false));
        }
        v
    }

    /// Like [`to_api_json`](Self::to_api_json) but flags the template as
    /// a writable user template (`is_system: false`, `is_owner: true`).
    /// Used for entries loaded from `config/docx-templates/user/`, which
    /// the template editor can modify and delete.
    pub fn to_api_json_user(&self) -> Value {
        let mut v = serde_json::to_value(self).unwrap_or(serde_json::json!({}));
        if let Value::Object(ref mut map) = v {
            map.insert("is_system".to_string(), Value::Bool(false));
            map.insert("is_owner".to_string(), Value::Bool(true));
        }
        v
    }

    /// Compose the system-prompt block that teaches an LLM how to
    /// write a document for this specific template. Derived
    /// **entirely** from the structured sidecar fields — margins,
    /// typography, section skeleton, required metadata — so a change
    /// to the JSON propagates to the prompt at the next restart
    /// without manual edits. The author can still append a free-form
    /// `prompt_md_extra` block for jurisdiction-specific tone notes.
    ///
    /// The output is the closing-formatter contract, including layout,
    /// section headers, and placeholder instructions.
    pub fn auto_generated_prompt_md(&self, locale: &str) -> String {
        let mut out = String::with_capacity(2048);
        let kind = self.display_name_for(locale);
        out.push_str(&format!("Generate a Word (.docx) document for: **{kind}**.\n\n"));

        // ── Source reference (so the LLM knows where the spec lives).
        if let Some(src) = &self.source_reference {
            out.push_str(&format!("Authoritative spec: {src}\n\n"));
        }

        // ── Layout block.
        out.push_str("LAYOUT (rendered automatically by the docx engine — do NOT include manual page-setup instructions in your output):\n");
        out.push_str(&format!(
            "- Paper: {} {}\n",
            self.paper.size, self.paper.orientation
        ));
        if self.paper.format != "standard" {
            out.push_str(&format!("- Special format: {}\n", self.paper.format));
        }
        out.push_str(&format!(
            "- Margins (cm): top {} / right {} / bottom {} / left {}\n",
            self.margins_cm.top,
            self.margins_cm.right,
            self.margins_cm.bottom,
            self.margins_cm.left,
        ));
        out.push_str(&format!(
            "- Body font: {} {}pt, line spacing {}, alignment {}\n",
            self.typography.body_font,
            self.typography.body_size_pt,
            self.typography.line_spacing,
            self.typography.alignment,
        ));
        if let Some(f) = &self.footnotes {
            out.push_str(&format!(
                "- Footnotes: {} {}pt, line spacing {}\n",
                f.font, f.size_pt, f.line_spacing
            ));
        }
        out.push('\n');

        // ── Placeholder convention declared by the template.
        out.push_str(&format!(
            "PLACEHOLDERS: use the `{}` convention — e.g. `[NOME]`, \
             `[DATA]`, `[PARTE_ASSISTITA.CF]`. Tokens are uppercase \
             with `_` and `.` allowed. The docx engine substitutes \
             every `[NAME]` against a metadata bag at render time. \
             Tokens that don't match a bag key are left verbatim in \
             the final document so the user sees the gap during \
             proofread.\n\n",
            self.placeholder_syntax,
        ));

        // ── Required metadata that the call to generate_docx must
        //    carry. Universal fields (LUOGO, DATA, MITTENTE, etc.)
        //    are inherited and always required — they're not listed
        //    here because every template needs them.
        if !self.required_metadata.is_empty() {
            out.push_str("REQUIRED METADATA (must be present in the `metadata` argument when calling `generate_docx`):\n");
            for field in &self.required_metadata {
                if let Some(hint) = self.field_prompts.get(field) {
                    out.push_str(&format!("- `{field}` — {hint}\n"));
                } else {
                    out.push_str(&format!("- `{field}`\n"));
                }
            }
            out.push('\n');
        }

        // ── Section skeleton (the structural blueprint).
        if !self.section_skeleton.is_empty() {
            out.push_str("SECTION SKELETON (emit sections in this order, using Markdown headings for titles):\n");
            for entry in &self.section_skeleton {
                let title = entry.title.as_deref().unwrap_or("");
                let render = entry.render.as_deref().unwrap_or("");
                let rep = if entry.repeating { " [REPEATING BLOCK]" } else { "" };
                let label = if !title.is_empty() {
                    format!("**{title}**{rep}")
                } else if !render.is_empty() {
                    format!("literal: `{render}`{rep}")
                } else {
                    format!("`{}`{rep}", entry.id)
                };
                out.push_str(&format!("- {label}"));
                if let Some(g) = &entry.guidance {
                    out.push_str(&format!(" — {g}"));
                }
                out.push('\n');
            }
            out.push('\n');
        }

        // ── Character limits (D.M. 110/2023 — atti difensivi only,
        //    but the field is generic).
        if let Some(limits) = &self.character_limits {
            out.push_str("CHARACTER LIMITS (apply by `atto_type` value):\n");
            let mut entries: Vec<(&String, &u64)> = limits.by_atto_type.iter().collect();
            entries.sort_by_key(|(k, _)| k.as_str());
            for (k, v) in entries {
                out.push_str(&format!("- `{k}`: max {v} characters\n"));
            }
            out.push_str("Exceeding the limit produces no invalidity but may be sanctioned by the judge — self-moderate.\n\n");
        }

        // ── Author override block (free-form tone notes).
        if let Some(extra) = &self.prompt_md_extra {
            out.push_str("ADDITIONAL AUTHOR NOTES:\n");
            out.push_str(extra.trim());
            out.push_str("\n\n");
        }

        out.push_str(&format!(
            "When ready, call `generate_docx(template_id=\"{}\", body_md=..., metadata=...)`. \
             Do NOT include layout instructions in the body — the engine handles them. \
             Write Markdown body content, no front-matter.\n",
            self.id,
        ));

        out
    }
}

// ─────────────────────────────────────────────────────────────────────
// Loader
// ─────────────────────────────────────────────────────────────────────

/// Walk every JSON file in `dir` (one level of subdirectory recursion
/// for domain folders), parse each as a `DocxTemplate`, validate
/// minimal invariants. Broken files are skipped with a `tracing::warn`
/// — one bad template doesn't take down the rest.
pub fn load_docx_templates(dir: &Path) -> Result<Vec<DocxTemplate>> {
    let mut out: Vec<DocxTemplate> = Vec::new();
    let files = match super::collect_json_files(dir) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::info!(
                "[docx-templates] directory {} not found; no templates loaded",
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
                    "[docx-templates] skip {} (read error): {}",
                    path.display(),
                    e
                );
                continue;
            }
        };
        match serde_json::from_slice::<DocxTemplate>(&bytes) {
            Ok(t) => {
                if let Err(reason) = validate(&t) {
                    tracing::warn!(
                        "[docx-templates] skip {}: {reason}",
                        path.display(),
                    );
                    continue;
                }
                tracing::info!(
                    "[docx-templates] loaded {} (domain={}, locale={}, L={})",
                    t.id,
                    t.domain,
                    t.locale,
                    t.automation_level,
                );
                out.push(t);
            }
            Err(e) => {
                tracing::warn!(
                    "[docx-templates] skip {} (parse error): {}",
                    path.display(),
                    e
                );
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

/// Minimum invariants every loaded template must satisfy. Public so the
/// `/docx-templates/save` write route can reject a malformed user
/// template before it ever touches disk.
pub fn validate(t: &DocxTemplate) -> Result<(), String> {
    if t.id.is_empty() {
        return Err("empty id".into());
    }
    if t.display_name.is_empty() {
        return Err(format!("template {}: display_name map is empty", t.id));
    }
    if !crate::domain::is_valid(&t.domain) {
        return Err(format!(
            "template {}: domain {} not in canonical set",
            t.id, t.domain
        ));
    }
    for d in &t.also_applicable_to {
        if !crate::domain::is_valid(d) {
            return Err(format!(
                "template {}: also_applicable_to entry {} not in canonical domain set",
                t.id, d
            ));
        }
        if d == &t.domain {
            return Err(format!(
                "template {}: also_applicable_to redundantly lists primary domain {}",
                t.id, d
            ));
        }
    }
    if !matches!(t.automation_level.as_str(), "L1" | "L2" | "L3" | "L4") {
        return Err(format!(
            "template {}: automation_level {} not in [L1,L2,L3,L4]",
            t.id, t.automation_level
        ));
    }
    if t.paper.format == "uso_bollo" && t.uso_bollo.is_none() {
        return Err(format!(
            "template {}: paper.format=uso_bollo requires uso_bollo block",
            t.id
        ));
    }
    if !matches!(t.placeholder_syntax.as_str(), "square_brackets" | "docproperty" | "jinja") {
        return Err(format!(
            "template {}: placeholder_syntax {} unsupported",
            t.id, t.placeholder_syntax
        ));
    }
    Ok(())
}

/// Synthetic English sidecar shared only by unit tests; never registered at runtime.
#[cfg(test)]
pub(crate) fn test_template() -> DocxTemplate {
    serde_json::from_str(r#"{
        "id": "test/project-update",
        "display_name": { "en": "Project update" },
        "category": "legal", "domain": "legal", "locale": "en",
        "paper": { "size": "A4" },
        "margins_cm": { "top": 2.5, "right": 2.5, "bottom": 2.5, "left": 3.0 },
        "typography": { "body_font": "Calibri", "body_size_pt": 11.0, "line_spacing": 1.15 },
        "style_map_baseline": {
            "body_text": "Body text", "section_heading": "Section heading",
            "citation": "Citation", "footnote": "Footnote"
        },
        "source_reference": "Test-only authoring specification",
        "required_metadata": ["PROJECT", "BUDGET", "DAYS", "SUMMARY", "OWNER"],
        "field_prompts": { "PROJECT": "Project name." },
        "section_skeleton": [
            { "id": "summary", "title": "Project summary", "guidance": "Summarize progress." },
            { "id": "items", "title": "Action items", "repeating": true }
        ]
    }"#).expect("test-only English template parses")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_template_json(id: &str, domain: &str) -> String {
        format!(
            r#"{{
                "id": "{id}",
                "display_name": {{ "en": "Test {id}" }},
                "category": "legal",
                "domain": "{domain}",
                "locale": "en",
                "paper": {{ "size": "A4" }},
                "margins_cm": {{ "top": 2.5, "right": 2.5, "bottom": 2.5, "left": 3.0 }},
                "typography": {{ "body_font": "Times New Roman", "body_size_pt": 12.0, "line_spacing": 1.5 }}
            }}"#
        )
    }

    #[test]
    fn parses_minimal_template() {
        let json = minimal_template_json("test/example", "legal");
        let t: DocxTemplate = serde_json::from_str(&json).expect("parse minimal");
        assert_eq!(t.id, "test/example");
        assert_eq!(t.domain, "legal");
        assert_eq!(t.automation_level, "L1"); // default
        assert_eq!(t.placeholder_syntax, "square_brackets"); // default
        assert_eq!(t.paper.format, "standard"); // default
        assert_eq!(t.style_map_baseline.len(), 4); // baseline always present
        assert!(t.style_map_baseline.contains_key("body_text"));
        assert!(t.style_map_baseline.contains_key("section_heading"));
        assert!(t.style_map_baseline.contains_key("citation"));
        assert!(t.style_map_baseline.contains_key("footnote"));
    }

    #[test]
    fn validate_rejects_invalid_domain() {
        let mut t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", "legal")).unwrap();
        t.domain = "made_up_domain".into();
        let err = validate(&t).unwrap_err();
        assert!(err.contains("domain"));
    }

    #[test]
    fn validate_rejects_invalid_automation_level() {
        let mut t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", "legal")).unwrap();
        t.automation_level = "L9".into();
        let err = validate(&t).unwrap_err();
        assert!(err.contains("automation_level"));
    }

    #[test]
    fn validate_rejects_uso_bollo_without_block() {
        let mut t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", "legal")).unwrap();
        t.paper.format = "uso_bollo".into();
        // uso_bollo block missing
        let err = validate(&t).unwrap_err();
        assert!(err.contains("uso_bollo"));
    }

    #[test]
    fn validate_accepts_uso_bollo_with_block() {
        let mut t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", "legal")).unwrap();
        t.paper.format = "uso_bollo".into();
        t.uso_bollo = Some(UsoBollo {
            line_spacing_pt_exact: 28.35,
            lines_per_facciata: 25,
            facciate_per_foglio: 4,
            mirror_margins: true,
            duplex: true,
            forbid_empty_lines: true,
            marginal_signature_required: true,
            signature_exclude_last_page: true,
        });
        assert!(validate(&t).is_ok());
    }

    #[test]
    fn display_name_falls_back_to_english_then_first() {
        let t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", "legal")).unwrap();
        assert_eq!(t.display_name_for("it"), "Test test/example");
        assert_eq!(t.display_name_for("en"), "Test test/example");
        // Locale we don't have → fallback to en
        assert_eq!(t.display_name_for("ja"), "Test test/example");
    }

    #[test]
    fn to_api_json_marks_system() {
        let t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", "legal")).unwrap();
        let v = t.to_api_json();
        assert_eq!(v["is_system"], serde_json::json!(true));
        assert_eq!(v["is_owner"], serde_json::json!(false));
        assert_eq!(v["id"], serde_json::json!("test/example"));
    }

    // ── matches_domain ──────────────────────────────────────────────

    fn template_with(domain: &str, also: Vec<&str>) -> DocxTemplate {
        let mut t: DocxTemplate =
            serde_json::from_str(&minimal_template_json("test/example", domain)).unwrap();
        t.also_applicable_to = also.iter().map(|s| s.to_string()).collect();
        t
    }

    #[test]
    fn matches_domain_none_means_everything() {
        let t = template_with("legal", vec![]);
        assert!(t.matches_domain(None));
        let t = template_with("finance", vec!["legal", "medical"]);
        assert!(t.matches_domain(None));
    }

    #[test]
    fn matches_domain_returns_true_for_primary() {
        let t = template_with("legal", vec![]);
        assert!(t.matches_domain(Some("legal")));
        assert!(!t.matches_domain(Some("finance")));
    }

    #[test]
    fn matches_domain_returns_true_for_also_applicable_entries() {
        let t = template_with("finance", vec!["legal", "medical", "ip"]);
        assert!(t.matches_domain(Some("finance")));
        assert!(t.matches_domain(Some("legal")));
        assert!(t.matches_domain(Some("medical")));
        assert!(t.matches_domain(Some("ip")));
        // Not in the cross-list.
        assert!(!t.matches_domain(Some("real_estate")));
        assert!(!t.matches_domain(Some("compliance")));
    }

    #[test]
    fn validate_rejects_invalid_entry_in_also_applicable_to() {
        let mut t = template_with("legal", vec![]);
        t.also_applicable_to.push("not_a_canonical_domain".into());
        let err = validate(&t).unwrap_err();
        assert!(err.contains("also_applicable_to"));
        assert!(err.contains("not_a_canonical_domain"));
    }

    #[test]
    fn validate_rejects_primary_domain_listed_redundantly() {
        // If primary == legal and also lists legal, that's noise —
        // surface it as an error so the author cleans up.
        let mut t = template_with("legal", vec![]);
        t.also_applicable_to.push("legal".into());
        let err = validate(&t).unwrap_err();
        assert!(err.contains("redundantly"));
        assert!(err.contains("legal"));
    }

    #[test]
    fn validate_accepts_valid_cross_domain_template() {
        let t = template_with("finance", vec!["legal", "medical", "ip", "compliance"]);
        assert!(validate(&t).is_ok());
    }

    #[test]
    fn cross_domain_template_matches_all_professional_domains() {
        let t = template_with("finance", vec!["legal", "medical", "ip", "compliance", "real_estate", "insurance"]);
        assert!(validate(&t).is_ok());
        for domain in ["finance", "legal", "medical", "ip", "compliance", "real_estate", "insurance"] {
            assert!(t.matches_domain(Some(domain)), "missing domain {domain}");
        }
    }

    #[test]
    fn domain_specific_template_excludes_other_domains() {
        for domain in ["real_estate", "compliance"] {
            let t = template_with(domain, vec![]);
            assert!(t.matches_domain(Some(domain)));
            assert!(!t.matches_domain(Some("legal")));
            assert!(!t.matches_domain(Some("finance")));
        }
    }

    #[test]
    fn loader_accepts_nested_test_sidecar_and_empty_registry() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_docx_templates(dir.path()).unwrap().is_empty());
        assert!(load_docx_templates(&dir.path().join("missing")).unwrap().is_empty());
        let nested = dir.path().join("test");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("project-update.json"), serde_json::to_vec(&test_template()).unwrap()).unwrap();
        let templates = load_docx_templates(dir.path()).expect("load synthetic sidecar");
        assert_eq!(templates.len(), 1);
        let t = &templates[0];
        assert_eq!(t.id, "test/project-update");
        assert!(validate(t).is_ok());
        assert_eq!(t.required_metadata, test_template().required_metadata);
        assert!(t.section_skeleton.iter().any(|s| s.id == "items" && s.repeating));
    }

    #[test]
    fn shipped_docx_registry_is_empty() {
        let dir = crate::presets::config_subdir("docx-templates");
        assert!(load_docx_templates(&dir).expect("load empty shipped registry").is_empty());
    }

    #[test]
    fn auto_generated_prompt_md_contains_layout_and_skeleton() {
        let prompt = test_template().auto_generated_prompt_md("en");
        for expected in ["Project update", "Paper: A4", "Calibri", "11pt",
            "Test-only authoring specification", "`PROJECT`", "`BUDGET`", "`DAYS`",
            "Project summary", "generate_docx", "square_brackets"] {
            assert!(prompt.contains(expected), "missing {expected}: {prompt}");
        }
        assert!(prompt.contains(r#"template_id="test/project-update""#));
    }

    #[test]
    fn auto_generated_prompt_md_marks_repeating_blocks() {
        assert!(test_template().auto_generated_prompt_md("en").contains("[REPEATING BLOCK]"));
    }

    // ── auto_generated_prompt_md edge cases on hand-rolled fixtures ──

    /// Build a maximally-minimal template — only the fields parse()
    /// would require — to test the "no optionals" path of the prompt
    /// generator.
    fn bare_template() -> DocxTemplate {
        let json = minimal_template_json("test/bare", "legal");
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn prompt_md_omits_sections_for_empty_optionals() {
        // A bare template has no source_reference, no
        // character_limits, no section_skeleton, no required_metadata,
        // no prompt_md_extra. The generator must omit those headers,
        // not emit empty ones.
        let t = bare_template();
        let prompt = t.auto_generated_prompt_md("en");
        // Headers absent
        assert!(!prompt.contains("Authoritative spec:"));
        assert!(!prompt.contains("REQUIRED METADATA"));
        assert!(!prompt.contains("SECTION SKELETON"));
        assert!(!prompt.contains("CHARACTER LIMITS"));
        assert!(!prompt.contains("ADDITIONAL AUTHOR NOTES"));
        // The closing instruction is always present.
        assert!(prompt.contains("generate_docx"));
    }

    #[test]
    fn prompt_md_emits_character_limits_sorted() {
        let mut t = bare_template();
        let mut limits = std::collections::HashMap::new();
        limits.insert("notes".to_string(), 10000u64);
        limits.insert("application".to_string(), 80000u64);
        limits.insert("motion".to_string(), 50000u64);
        t.character_limits = Some(CharacterLimits { by_atto_type: limits });

        let prompt = t.auto_generated_prompt_md("en");
        assert!(prompt.contains("CHARACTER LIMITS"));
        assert!(prompt.contains("`application`: max 80000"));
        assert!(prompt.contains("`motion`: max 50000"));
        assert!(prompt.contains("`notes`: max 10000"));
        // Alphabetic order: 'a' < 'm' < 'n'. Find each substring and
        // assert their relative position.
        let pos_a = prompt.find("application").unwrap();
        let pos_m = prompt.find("motion").unwrap();
        let pos_n = prompt.find("notes").unwrap();
        assert!(pos_a < pos_m && pos_m < pos_n, "limits must be alphabetically sorted");
    }

    #[test]
    fn prompt_md_appends_author_override_when_present() {
        let mut t = bare_template();
        t.prompt_md_extra =
            Some("Use plain English and concise headings.".into());
        let prompt = t.auto_generated_prompt_md("en");
        assert!(prompt.contains("ADDITIONAL AUTHOR NOTES"));
        assert!(prompt.contains("Use plain English"));
    }

    #[test]
    fn prompt_md_handles_section_skeleton_with_literal_render() {
        // A section with `render` (literal text, e.g. "* * *") and no
        // title should be rendered as `literal: ...` so the LLM
        // knows to emit the verbatim string.
        let mut t = bare_template();
        t.section_skeleton = vec![
            SectionSkeletonEntry {
                id: "facts".into(),
                title: Some("FACTS".into()),
                render: None,
                guidance: Some("Statement of facts.".into()),
                repeating: false,
            },
            SectionSkeletonEntry {
                id: "separator".into(),
                title: None,
                render: Some("* * *".into()),
                guidance: None,
                repeating: false,
            },
        ];
        let prompt = t.auto_generated_prompt_md("en");
        assert!(prompt.contains("**FACTS**"));
        assert!(prompt.contains("Statement of facts."));
        assert!(prompt.contains("literal: `* * *`"));
    }

    #[test]
    fn prompt_md_field_prompts_attached_to_required_metadata() {
        let mut t = bare_template();
        t.required_metadata = vec!["CLIENT".into(), "AMOUNT".into()];
        t.field_prompts.insert(
            "CLIENT".into(),
            "Client name.".into(),
        );
        // AMOUNT without a field_prompts entry — should still appear
        // in the prompt, just without the hint.
        let prompt = t.auto_generated_prompt_md("en");
        assert!(prompt.contains("`CLIENT` — Client name."));
        // AMOUNT line: id present, no em-dash hint.
        let amount_line = prompt
            .lines()
            .find(|l| l.contains("`AMOUNT`"))
            .expect("AMOUNT listed");
        assert!(!amount_line.contains(" — "), "AMOUNT line should not carry a hint dash");
    }

    #[test]
    fn prompt_md_uses_locale_for_display_name() {
        let mut t = bare_template();
        t.display_name.insert("fr".to_string(), "Alternate display name".to_string());
        let fallback = t.auto_generated_prompt_md("ja");
        assert!(fallback.contains("Test test/bare"));
        let localized = t.auto_generated_prompt_md("fr");
        assert!(localized.contains("Alternate display name"));
    }

    #[test]
    fn prompt_md_mentions_uso_bollo_special_format() {
        let mut t = bare_template();
        t.paper.format = "uso_bollo".into();
        t.uso_bollo = Some(UsoBollo {
            line_spacing_pt_exact: 28.35,
            lines_per_facciata: 25,
            facciate_per_foglio: 4,
            mirror_margins: true,
            duplex: true,
            forbid_empty_lines: true,
            marginal_signature_required: true,
            signature_exclude_last_page: true,
        });
        let prompt = t.auto_generated_prompt_md("en");
        // Special-format line surfaces the variant name.
        assert!(prompt.contains("Special format: uso_bollo"));
    }

    #[test]
    fn character_limits_parses_flexible_map() {
        let json = r#"{
            "id": "test/brief",
            "display_name": { "en": "Brief" },
            "category": "legal",
            "domain": "legal",
            "locale": "en",
            "paper": { "size": "A4" },
            "margins_cm": { "top": 3.0, "right": 2.0, "bottom": 2.5, "left": 3.5 },
            "typography": { "body_font": "Times New Roman", "body_size_pt": 12.0, "line_spacing": 1.5 },
            "character_limits": {
                "application": 80000,
                "motion": 50000,
                "notes": 10000
            }
        }"#;
        let t: DocxTemplate = serde_json::from_str(json).expect("parse");
        let limits = t.character_limits.expect("has limits");
        assert_eq!(limits.by_atto_type["application"], 80000);
        assert_eq!(limits.by_atto_type["motion"], 50000);
    }
}
