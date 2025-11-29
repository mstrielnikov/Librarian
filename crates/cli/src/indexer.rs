use anyhow::{Context, Result};
use arrow_array::{RecordBatch, RecordBatchIterator};
use arrow_json::reader::infer_json_schema;
use graphodoc_core::{Doc, graph};
use lancedb::connection::Connection;
use pulldown_cmark::{Parser, Event};
use regex::Regex;
use std::{fs, path::Path, sync::Arc, io::{Cursor, Seek, SeekFrom}};
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;
use crate::Cli;

pub async fn run_indexer(cli: &Cli) -> Result<()> {
    println!("🔍 Scanning directory: {:?}", cli.dir);

    // 1. Scan Docs
    let docs = scan_docs(&cli.dir)?;
    println!("📄 Found {} markdown documents.", docs.len());
    if docs.is_empty() { return Ok(()); }

    // 2. Build Graph
    println!("🕸️ Building Graph...");
    let graph_data = graph::build_graph_from_docs(&docs);
    println!("✅ Graph built: {} nodes, {} edges.", graph_data.nodes.len(), graph_data.edges.len());

    // 3. Save to DB
    let db = lancedb::connect(cli.db.to_str().unwrap_or("./mdkb_data"))
        .execute().await?;

    save_table(&db, "documents", &docs, cli.rebuild).await?;
    save_table(&db, "nodes", &graph_data.nodes, cli.rebuild).await?;
    save_table(&db, "edges", &graph_data.edges, cli.rebuild).await?;

    println!("🚀 Indexing complete!");
    Ok(())
}

fn scan_docs(root: &Path) -> Result<Vec<Doc>> {
    let mut docs = Vec::new();
    let wiki_link_re = Regex::new(r"\[\[(.*?)\]\]").unwrap();
    let tag_re = Regex::new(r"#(\w+)").unwrap();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read {:?}", entry.path()))?;

            let rel_path = entry.path().strip_prefix(root).unwrap_or(entry.path());
            let title = entry.path().file_stem().unwrap().to_string_lossy().to_string();

            // FIX: Generate Hex String ID
            let hash = xxh3_64(rel_path.to_string_lossy().as_bytes());
            let id = format!("{:016x}", hash);

            let (text, keywords) = extract_text_and_keywords(&content);
            let wiki_links: Vec<String> = wiki_link_re.captures_iter(&content).map(|c| c[1].to_string()).collect();
            let tags: Vec<String> = tag_re.captures_iter(&content).map(|c| c[1].to_string()).collect();

            docs.push(Doc { id, path: rel_path.to_string_lossy().to_string(), title, content, text, wiki_links, tags, keywords });
        }
    }
    Ok(docs)
}

// ... (Helper functions 'extract_text_and_keywords' and 'save_table' remain the same as previous steps)
// Just ensure save_table uses 'db.drop_table(table_name, &Vec::new())'
// Re-paste them below if you need the full file content.

fn extract_text_and_keywords(md_content: &str) -> (String, Vec<String>) {
    let parser = Parser::new(md_content);
    let mut text_acc = String::new();
    for event in parser {
        match event {
            Event::Text(t) | Event::Code(t) => { text_acc.push_str(&t); text_acc.push(' '); },
            _ => {}
        }
    }
    let keywords: Vec<String> = text_acc.split_whitespace()
        .map(|s| s.to_lowercase())
        .filter(|s| s.len() > 3 && s.chars().all(char::is_alphanumeric))
        .collect::<std::collections::HashSet<_>>().into_iter().collect();
    (text_acc, keywords)
}

async fn save_table<T: serde::Serialize + 'static>(db: &Connection, table_name: &str, data: &[T], drop_existing: bool) -> Result<()> {
    if data.is_empty() { return Ok(()); }
    let mut ndjson = String::new();
    for item in data { ndjson.push_str(&serde_json::to_string(item)?); ndjson.push('\n'); }

    let mut cursor = Cursor::new(ndjson.as_bytes());
    let (schema, _) = infer_json_schema(&mut cursor, None)?;
    let schema_ref = Arc::new(schema);

    cursor.seek(SeekFrom::Start(0))?;
    let batch = arrow_json::ReaderBuilder::new(schema_ref.clone()).build(cursor)?.next().context("No batch")??;

    if db.table_names().execute().await?.contains(&table_name.to_string()) {
        if drop_existing {
            db.drop_table(table_name, &Vec::new()).await?;
            db.create_table(table_name, RecordBatchIterator::new(vec![Ok(batch)], schema_ref)).execute().await?;
        } else {
            db.open_table(table_name).execute().await?
                .add(RecordBatchIterator::new(vec![Ok(batch)], schema_ref)).execute().await?;
        }
    } else {
        db.create_table(table_name, RecordBatchIterator::new(vec![Ok(batch)], schema_ref)).execute().await?;
    }
    println!("💾 Saved {} records to '{}'", data.len(), table_name);
    Ok(())
}
