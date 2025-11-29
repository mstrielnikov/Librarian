pub mod model;
pub mod parser;
pub mod graph;
pub mod store;

use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;

// Public facade function
pub async fn process_knowledge_base(
    input_dir: PathBuf,
    db_path: PathBuf,
    rebuild: bool,
    export_json: bool
) -> Result<()> {

    // 1. Ingest Documents
    let paths: Vec<PathBuf> = WalkDir::new(&input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        .map(|e| e.into_path())
        .collect();

    let docs: Vec<model::Doc> = paths
        .par_iter()
        .filter_map(|p| match parser::parse_file(p) {
            Ok(doc) => Some(doc),
            Err(e) => {
                eprintln!("Error parsing {}: {}", p.display(), e);
                None
            }
        })
        .collect();

    // 2. Build Graph (In Memory)
    let graph = graph::build_graph_from_docs(&docs);

    // 3. Optional Visualization Output
    if export_json {
        let json_out = serde_json::to_string(&graph)?;
        println!("{}", json_out);
    }

    // 4. Persistence
    let store = store::VectorStore::connect(&db_path).await?;

    if rebuild {
        store.reset().await?;
    }

    store.save_graph(&docs, &graph).await?;

    Ok(())
}
