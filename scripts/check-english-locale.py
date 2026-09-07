"""Static English-only API/prompt gate for snapshots without a Rust toolchain."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1] / "src"


class EnglishLocaleTests(unittest.TestCase):
    def test_api_reads_english_and_restricts_writes(self):
        source = (ROOT / "routes/user.rs").read_text()
        getter = source.split("async fn get_locale(", 1)[1].split("#[derive(Deserialize)]", 1)[0]
        self.assertIn('json!({ "locale": "en" })', getter)
        setter = source.split("async fn update_locale(", 1)[1].split("// ----", 1)[0]
        self.assertIn('supported_ui_locale(&body.locale)', setter)

    def test_prompt_resolution_never_loads_legacy_locale(self):
        source = (ROOT / "presets/system_prompt.rs").read_text()
        resolver = source.split("pub fn resolve(", 1)[1].split("pub fn default_country_for_locale", 1)[0]
        self.assertIn('try_read(&root, "en", &domain_safe)', resolver)
        self.assertNotIn('FALLBACK_LOCALES', source)
        self.assertNotIn('"it" => "Italian"', source)


if __name__ == "__main__":
    unittest.main()
