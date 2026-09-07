//! Run without the desktop: cargo run --example apac_probe -- <manifest-dir> [corpus-id identifier]
use anyhow::{Context, Result, ensure};
use mike::corpora::{LegalCorpusAdapter, manifest_adapter::ManifestAdapter, plugin::CorpusPlugin};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    ensure!(args.len() >= 2, "expected manifest directory");
    let mut plugins = Vec::new();
    for entry in std::fs::read_dir(&args[1])? {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") { continue; }
        let plugin: CorpusPlugin = serde_json::from_slice(&std::fs::read(&path)?)
            .with_context(|| format!("parse {}", path.display()))?;
        plugin.validate().with_context(|| format!("validate {}", path.display()))?;
        println!("VALID {} available={}", plugin.id, plugin.available);
        plugins.push(plugin);
    }
    println!("VALIDATED {} manifests", plugins.len());
    if args.len() >= 4 {
        let plugin = plugins.iter().find(|p| p.id == args[2]).context("unknown corpus")?;
        let adapter = ManifestAdapter::try_from_plugin(plugin).context("not HTTP adapter")?;
        let doc = adapter.fetch(&args[3], None, false).await?;
        ensure!(!doc.bytes.is_empty(), "empty document");
        println!("FETCH title={} bytes={} mime={} url={}", doc.title, doc.bytes.len(), doc.mime, doc.source_url);
        if doc.mime.starts_with("text/") {
            let text = String::from_utf8_lossy(&doc.bytes);
            println!("BEGIN {}", text.chars().take(300).collect::<String>());
            println!("END {}", text.chars().rev().take(300).collect::<String>().chars().rev().collect::<String>());
        }
    }
    Ok(())
}
