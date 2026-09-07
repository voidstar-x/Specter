"""Check Specter's retired connector and English-only asset boundaries.

Run from any directory with Python 3: python scripts/check-cleanup.py
Build and runtime tests remain separate gates.
"""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class CleanupTests(unittest.TestCase):
    def test_retired_connectors_have_no_frontend_references(self):
        retired = re.compile(r"eurlex|EUR-Lex|italianLegal|ItalianLegal|isItalian")
        violations = []
        for path in (ROOT / "frontend/src").rglob("*"):
            if path.is_file():
                for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
                    if retired.search(line):
                        violations.append(f"{path.relative_to(ROOT)}:{number}")
        self.assertEqual(violations, [], "Retired connector frontend references remain")

    def test_only_english_ui_catalog_is_shipped(self):
        catalogs = sorted(p.name for p in (ROOT / "frontend/locales").glob("*.json"))
        self.assertEqual(catalogs, ["en.json"])


if __name__ == "__main__":
    unittest.main()
