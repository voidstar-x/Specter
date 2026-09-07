"""Dependency-free static regression gate; Rust tests verify runtime serde behavior.

Run: python scripts/check-corpora-cleanup.py
"""
import json
from pathlib import Path
import re
import unittest

REPO = Path(__file__).resolve().parents[1]
ROOT = REPO / "src"


class CorpusRetirementTests(unittest.TestCase):
    def test_only_runnable_http_strategy_is_accepted(self):
        source = (ROOT / "corpora/plugin.rs").read_text()
        enum = source.split("pub enum CorpusStrategy {", 1)[1].split("\n}", 1)[0]
        variants = re.findall(r"^    ([A-Z][A-Za-z0-9_]*)\s*[({]", enum, re.M)
        self.assertEqual(variants, ["HttpFetchPerId"])
        self.assertNotIn("KNOWN_BUILTINS", source)

    def test_no_retired_adapter_registrations_or_import_endpoints(self):
        registry = (ROOT / "corpora/manifest_adapter.rs").read_text()
        self.assertNotIn("CorpusStrategy::Builtin", registry)
        routes = (ROOT / "routes/corpora.rs").read_text()
        self.assertNotRegex(routes, r'\.route\("/\{id\}/import(?:"|-status"|-progress")')
        state = (ROOT / "db/mod.rs").read_text()
        self.assertNotIn("corpus_import_progress", state)

    def test_all_eight_apac_sources_keep_http_and_native_languages(self):
        files = sorted((REPO / "config/corpora-plugins").glob("*.json"))
        manifests = [json.loads(path.read_text()) for path in files]
        self.assertEqual({m["id"] for m in manifests}, {
            "au-federalregister", "id-peraturan", "jp-egov", "kr-lawinfo",
            "my-lom", "sg-statutes", "th-royalgazette", "vn-legal",
        })
        self.assertEqual(len(manifests), 8)
        for manifest in manifests:
            self.assertEqual(manifest["strategy"]["kind"], "http-fetch-per-id")
            self.assertIn(manifest["default_language"], manifest["languages"])
            self.assertFalse(manifest.get("capabilities", {}).get("bulk_import", False))
        shapes = {m["strategy"]["search_by_id"]["shape"] for m in manifests}
        self.assertIn("direct-pdf", shapes)
        self.assertIn("rest-html", shapes)


if __name__ == "__main__":
    unittest.main()
