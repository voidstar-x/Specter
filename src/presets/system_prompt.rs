// Copyright (c) 2026 Specter contributors. Licensed under AGPL-3.0-only.

//! Domain-aware system-prompt **prologue**. Read once at chat-turn
//! time from `config/system-prompts/<locale>/<domain>.md` and
//! prepended to `MRUST_SYSTEM_PROMPT` so the assistant boots with a
//! professional-vertical persona before the generic Mike tool-use /
//! citation rules kick in.
//!
//! Specter always loads `en/<domain>.md`; legacy stored UI locales do
//! not select a different language or imply a jurisdiction. The English
//! domain prompts provide practice-specific context and jurisdiction guidance.
//!
//! Directory discovery: `MRUST_SYSTEM_PROMPTS_DIR`, CWD ancestors,
//! then executable ancestors. Missing English prompts return `None`.

use std::path::{Path, PathBuf};

/// Locate the `config/system-prompts/` root directory. Mirrors the
/// `presets_dir` / `config_subdir` pattern in `crate::presets` so the
/// installed-MSI layout (`<install>/config/system-prompts/`) and the
/// dev workspace layout (`<repo>/config/system-prompts/`) both work
/// without an env-var override.
fn root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MRUST_SYSTEM_PROMPTS_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(found) = walk_for_root(&cwd) {
            return found;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(found) = walk_for_root(&exe) {
            return found;
        }
    }
    PathBuf::from("./config/system-prompts")
}

fn walk_for_root(start: &Path) -> Option<PathBuf> {
    for anc in start.ancestors() {
        let candidate = anc.join("config").join("system-prompts");
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
}

/// Load the English domain prompt, regardless of a legacy UI locale.
/// Invalid path segments are still rejected before filesystem access.
pub fn resolve(locale: &str, domain: &str) -> Option<String> {
    let root = root_dir();
    let domain_safe = sanitize_segment(domain)?;
    sanitize_segment(locale)?;
    try_read(&root, "en", &domain_safe)
}

/// UI language does not determine the applicable legal jurisdiction.
pub fn default_country_for_locale(_locale: &str) -> &'static str {
    "unspecified (ask the user)"
}

/// Specter's sole working language, including for legacy saved locales.
pub fn language_name_for_locale(_locale: &str) -> &'static str {
    "English"
}

/// Assemble the full prologue section that gets prepended to
/// `MRUST_SYSTEM_PROMPT`. Wraps the per-domain `.md` body in a
/// metadata header (Domain / Working language / Default country) and
/// a country-disambiguation reminder. Returns an empty string when
/// nothing meaningful can be assembled (no `.md` found AND no domain
/// known) — the caller then skips the section entirely.
pub fn assemble_prologue(locale: &str, domain: &str) -> String {
    let body = resolve(locale, domain).unwrap_or_default();
    let lang = language_name_for_locale(locale);
    let country = default_country_for_locale(locale);
    let mut out = String::new();
    out.push_str(
        "=== Domain context (read this first, it sets your role for this chat) ===\n",
    );
    out.push_str(&format!("Domain: {domain}\n"));
    out.push_str(&format!("Working language: {lang}\n"));
    out.push_str(&format!("Default country / jurisdiction: {country}\n\n"));
    if body.is_empty() {
        out.push_str(
            "No domain-specific guidance is available; behave as a generic professional \
             assistant for this vertical. Cite sources when relevant, defer to the user on \
             jurisdiction-specific decisions.\n",
        );
    } else {
        out.push_str(&body);
        out.push('\n');
    }
    out.push_str(
        "\nCountry disambiguation: If the user's request involves a country, regulation, \
         or legal/medical/professional framework that does not match the default above, \
         ASK the user which country / jurisdiction applies BEFORE giving jurisdiction-\
         specific advice. Do not silently assume.\n",
    );
    out
}

fn try_read(root: &Path, locale: &str, domain: &str) -> Option<String> {
    let path = root.join(locale).join(format!("{domain}.md"));
    if !path.is_file() {
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            let t = s.trim().to_string();
            if t.is_empty() {
                None
            } else {
                Some(t)
            }
        }
        Err(e) => {
            tracing::warn!(
                "[system-prompts] failed to read {}: {e}",
                path.display()
            );
            None
        }
    }
}

/// Guard against `..` / absolute path injection through user-supplied
/// locale or domain. Accepts only `[a-zA-Z0-9_-]+`; anything else
/// returns `None` and the caller treats the file as missing.
fn sanitize_segment(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    if t.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        Some(t.to_ascii_lowercase())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::sync::{Mutex, MutexGuard};
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct PromptTree {
        _tmp: TempDir,
        previous_dir: Option<OsString>,
        _lock: MutexGuard<'static, ()>,
    }

    impl Drop for PromptTree {
        fn drop(&mut self) {
            // Keep the lock held while restoring the caller's environment,
            // including when a test assertion panics.
            unsafe {
                match &self.previous_dir {
                    Some(dir) => std::env::set_var("MRUST_SYSTEM_PROMPTS_DIR", dir),
                    None => std::env::remove_var("MRUST_SYSTEM_PROMPTS_DIR"),
                }
            }
        }
    }

    fn make_tree(files: &[(&str, &str, &str)]) -> PromptTree {
        let lock = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let tmp = TempDir::new().unwrap();
        for (locale, domain, body) in files {
            let dir = tmp.path().join(locale);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(format!("{domain}.md")), body).unwrap();
        }
        let previous_dir = std::env::var_os("MRUST_SYSTEM_PROMPTS_DIR");
        // Serialize this module's environment overrides and resolver calls.
        // The returned guard holds the lock for the entire test fixture lifetime.
        unsafe {
            std::env::set_var("MRUST_SYSTEM_PROMPTS_DIR", tmp.path());
        }
        PromptTree {
            _tmp: tmp,
            previous_dir,
            _lock: lock,
        }
    }

    #[test]
    fn resolve_uses_english_even_when_legacy_locale_is_present() {
        let _tmp = make_tree(&[
            ("it", "medical", "ITALIANO"),
            ("en", "medical", "ENGLISH"),
        ]);
        assert_eq!(resolve("it", "medical").as_deref(), Some("ENGLISH"));
    }

    #[test]
    fn resolve_does_not_fall_back_to_legacy_locale() {
        let _tmp = make_tree(&[("it", "medical", "ITALIANO")]);
        assert!(resolve("it", "medical").is_none());
    }

    #[test]
    fn resolve_uses_english_for_other_ui_locales() {
        let _tmp = make_tree(&[("en", "medical", "ENGLISH")]);
        assert_eq!(resolve("de", "medical").as_deref(), Some("ENGLISH"));
    }

    #[test]
    fn resolve_returns_none_when_domain_missing_everywhere() {
        let _tmp = make_tree(&[("en", "medical", "ENGLISH")]);
        assert!(resolve("it", "finance").is_none());
    }

    #[test]
    fn resolve_rejects_path_traversal() {
        let _tmp = make_tree(&[("en", "medical", "ENGLISH")]);
        assert!(resolve("../etc", "medical").is_none());
        assert!(resolve("it", "../passwd").is_none());
    }

    #[test]
    fn assemble_prologue_wraps_body() {
        let _tmp = make_tree(&[("en", "medical", "BODY")]);
        let p = assemble_prologue("it", "medical");
        assert!(p.contains("Domain: medical"));
        assert!(p.contains("Working language: English"));
        assert!(p.contains("Default country / jurisdiction: unspecified (ask the user)"));
        assert!(p.contains("BODY"));
        assert!(p.contains("Country disambiguation"));
    }

    #[test]
    fn assemble_prologue_falls_back_when_md_missing() {
        let _tmp = make_tree(&[("en", "medical", "BODY")]);
        let p = assemble_prologue("it", "ip");
        assert!(p.contains("Domain: ip"));
        assert!(p.contains("No domain-specific guidance"));
        assert!(p.contains("Country disambiguation"));
    }

    #[test]
    fn default_country_for_locale_known_locales() {
        for locale in ["it", "fr", "de", "es", "pt", "en"] {
            assert_eq!(default_country_for_locale(locale), "unspecified (ask the user)");
            assert_eq!(language_name_for_locale(locale), "English");
        }
        assert!(default_country_for_locale("en").contains("ask"));
    }
}
