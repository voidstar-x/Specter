//! Integration tests for the standalone Markdown-to-DOCX renderer.
//!
//! The English sidecar and project-update data below are synthetic and test-only.
//! No installed template registry, external documents, model, or API is required.
//! Covers sidecar deserialization, OOXML packaging, placeholder substitution,
//! missing-field reporting, typography, Markdown, Unicode, and XML escaping.

use std::collections::HashMap;
use std::io::Read;

use mike::docx;
use mike::presets::docx_template::DocxTemplate;
use quick_xml::events::Event;

/// Exercise the sidecar schema without depending on application configuration.
fn test_template() -> DocxTemplate {
    serde_json::from_str(
        r#"{
            "schema_version": 1,
            "id": "test/project-update",
            "display_name": { "en": "Project update" },
            "category": "report",
            "domain": "general",
            "locale": "en",
            "paper": { "size": "A4" },
            "margins_cm": { "top": 2.5, "right": 2.5, "bottom": 2.5, "left": 2.5 },
            "typography": {
                "body_font": "Calibri",
                "body_size_pt": 11.0,
                "line_spacing": 1.15
            },
            "style_map_baseline": {
                "body_text": "Project body",
                "section_heading": "Project heading",
                "citation": "Quotation",
                "footnote": "Footnote"
            },
            "required_metadata": ["PROJECT", "REFERENCE"]
        }"#,
    )
    .expect("test-only English sidecar should deserialize")
}

fn read_part(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .expect("rendered document should be a valid ZIP archive");
    let mut xml = String::new();
    archive
        .by_name(name)
        .unwrap_or_else(|err| panic!("missing OOXML part {name}: {err}"))
        .read_to_string(&mut xml)
        .expect("OOXML part should be readable UTF-8");
    xml
}

/// Parse XML and decode its text, rather than relying on raw-ampersand heuristics.
fn xml_text(xml: &str) -> String {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut text = String::new();
    loop {
        match reader.read_event().expect("OOXML should be well-formed") {
            Event::Text(value) => {
                text.push_str(&value.unescape().expect("XML text entities should decode"));
            }
            Event::Start(element) | Event::Empty(element) => {
                for attribute in element.attributes() {
                    attribute
                        .expect("XML attributes should be valid")
                        .unescape_value()
                        .expect("XML attribute entities should decode");
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    text
}

#[test]
fn renders_project_update_with_substitutions_and_sidecar_styles() {
    let template = test_template();
    let metadata: HashMap<String, String> = [
        ("PROJECT", "Sample workspace refresh"),
        ("REFERENCE", "DEMO-0042"),
        ("OWNER", "Taylor Example"),
        ("TEAM", "Sample operations team"),
        ("LOCATION", "Singapore"),
        ("DATE", "7 September 2026"),
        ("STATUS", "Ready for review"),
        ("BUDGET", "SGD 1,250.00"),
        ("DURATION", "Three weeks"),
        ("SUMMARY", "The sample workspace layout is ready for feedback."),
        ("NEXT_STEP", "Collect comments on the draft layout."),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value.to_string()))
    .collect();
    let body_md = r#"# Project update: [PROJECT]

Reference: [REFERENCE]. Prepared by [OWNER] for [TEAM].

[LOCATION], [DATE].

## Summary

[SUMMARY]

Status: **[STATUS]**. Planned duration: *[DURATION]*.

## Resources

The demonstration budget is [BUDGET]. This update records a fictional
workspace exercise, not a commitment or a request for payment.

## Next steps

[NEXT_STEP]
"#;
    let outcome = docx::render(&template, body_md, &metadata).expect("render should succeed");

    assert!(outcome.bytes.starts_with(b"PK\x03\x04"));
    for required in [
        "[Content_Types].xml",
        "_rels/.rels",
        "word/_rels/document.xml.rels",
        "word/styles.xml",
        "word/document.xml",
    ] {
        let xml = read_part(&outcome.bytes, required);
        assert!(!xml.is_empty(), "OOXML part {required} should not be empty");
        xml_text(&xml);
    }

    let doc_xml = read_part(&outcome.bytes, "word/document.xml");
    let text = xml_text(&doc_xml);
    for (key, value) in &metadata {
        assert!(text.contains(value), "substitution for {key} should survive rendering");
        assert!(!text.contains(&format!("[{key}]")), "{key} should be replaced");
    }
    assert!(
        outcome.unresolved_placeholders.is_empty(),
        "all body fields were supplied: {:?}",
        outcome.unresolved_placeholders
    );
    assert!(doc_xml.contains(r#"w:pStyle w:val="SectionHeading""#));
    assert!(doc_xml.contains("<w:b/>"), "strong Markdown should produce bold runs");
    assert!(doc_xml.contains("<w:i/>"), "emphasis should produce italic runs");

    let styles_xml = read_part(&outcome.bytes, "word/styles.xml");
    assert!(styles_xml.contains(r#"w:ascii="Calibri""#));
    assert!(
        styles_xml.contains(r#"<w:sz w:val="22"/>"#),
        "11pt body size should be emitted as 22 half-points"
    );
    assert!(styles_xml.contains(r#"<w:name w:val="Project body"/>"#));
    assert!(styles_xml.contains(r#"<w:name w:val="Project heading"/>"#));
    assert!(
        outcome.bytes.len() > 2_000,
        "rendered DOCX should not be an empty shell: {} bytes",
        outcome.bytes.len()
    );
}

#[test]
fn missing_field_is_reported_and_preserved_in_a_readable_document() {
    let template = test_template();
    let metadata = HashMap::from([("PROJECT".to_string(), "Sample workspace refresh".to_string())]);
    let outcome = docx::render(
        &template,
        "Project: [PROJECT]. Reference: [REFERENCE].",
        &metadata,
    )
    .expect("missing metadata should not block rendering");

    assert_eq!(outcome.unresolved_placeholders, vec!["REFERENCE".to_string()]);
    assert!(outcome.bytes.starts_with(b"PK\x03\x04"));
    let text = xml_text(&read_part(&outcome.bytes, "word/document.xml"));
    assert!(text.contains("Sample workspace refresh"));
    assert!(!text.contains("[PROJECT]"));
    assert!(text.contains("[REFERENCE]"), "missing token should remain visible");
}

#[test]
fn unicode_and_xml_special_characters_round_trip_through_the_package() {
    let template = test_template();
    let special = "A & B: 3 < 5 > 2; \"quoted\" and O'Neil";
    let unicode = "Zoë — € 1,250.00";
    let metadata = HashMap::from([
        ("PROJECT".to_string(), special.to_string()),
        ("REFERENCE".to_string(), unicode.to_string()),
    ]);
    let outcome = docx::render(
        &template,
        "# Café notes — résumé\n\n[PROJECT]\n\n[REFERENCE]\n\nDirect text: R&D, 2 < 4 > 1.",
        &metadata,
    )
    .expect("Unicode and XML-special text should render");

    assert!(outcome.unresolved_placeholders.is_empty());
    let doc_xml = read_part(&outcome.bytes, "word/document.xml");
    let text = xml_text(&doc_xml);
    assert!(text.contains(special), "all XML-special metadata should round-trip");
    assert!(text.contains(unicode), "Unicode metadata should round-trip");
    assert!(text.contains("Café notes — résumé"), "literal Unicode should survive");
    assert!(text.contains("Direct text: R&D, 2 < 4 > 1."));
    for entity in ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"] {
        assert!(doc_xml.contains(entity), "expected XML escaping for {entity}");
    }
    assert!(!doc_xml.contains("A & B"), "raw ampersands must be escaped");
    assert!(!doc_xml.contains("3 < 5"), "raw less-than signs must be escaped");
}
