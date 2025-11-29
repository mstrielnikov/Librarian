use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router, http::StatusCode,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use graphodoc_core::{GraphData, Node, Edge};
use futures::TryStreamExt;
use lancedb::query::ExecutableQuery;

struct AppState {
    db_path: PathBuf,
}

pub async fn start_server(port: u16, db_path: PathBuf) -> anyhow::Result<()> {
    let state = Arc::new(AppState { db_path });
    let web_dist = PathBuf::from("./crates/web/dist");

    let app = Router::new()
        .route("/api/graph", get(get_graph))
        .route("/api/doc/:id", get(get_doc))
        .nest_service("/", ServeDir::new(web_dist))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Graphodoc Web UI running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_graph(State(state): State<Arc<AppState>>) -> Result<Json<GraphData>, StatusCode> {
    // 1. Connect to DB
    let db = lancedb::connect(state.db_path.to_str().unwrap_or("./mdkb_data"))
        .execute()
        .await
        .map_err(|e| {
            eprintln!("❌ DB Connect Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 2. Fetch Nodes (Safe Helper)
    let nodes: Vec<Node> = fetch_table_safe(&db, "nodes").await;

    // 3. Fetch Edges (Safe Helper)
    let edges: Vec<Edge> = fetch_table_safe(&db, "edges").await;

    // Log success for debugging
    println!("✅ Served {} nodes and {} edges", nodes.len(), edges.len());

    Ok(Json(GraphData { nodes, edges }))
}

// Helper to handle table opening/querying errors safely
async fn fetch_table_safe<T: serde::de::DeserializeOwned>(db: &lancedb::Connection, table_name: &str) -> Vec<T> {
    use arrow_json::WriterBuilder;
    use arrow_json::writer::JsonArray;

    // Try to open the table
    let table = match db.open_table(table_name).execute().await {
        Ok(t) => t,
        Err(_) => {
            eprintln!("⚠️ Table '{}' not found (run indexer first)", table_name);
            return vec![];
        }
    };

    // Try to query the table
    let batches = match table.query().execute().await {
        Ok(stream) => match stream.try_collect::<Vec<_>>().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("❌ Error collecting batches for '{}': {}", table_name, e);
                return vec![];
            }
        },
        Err(e) => {
            eprintln!("❌ Error executing query for '{}': {}", table_name, e);
            return vec![];
        }
    };

    if batches.is_empty() {
        return vec![];
    }

    // Serialize Arrow to JSON
    let mut buf = Vec::new();
    let mut writer = WriterBuilder::new().build::<_, JsonArray>(&mut buf);
    for batch in batches {
        if let Err(e) = writer.write(&batch) {
            eprintln!("❌ Error writing batch for '{}': {}", table_name, e);
        }
    }
    if let Err(e) = writer.finish() {
        eprintln!("❌ Error finishing JSON for '{}': {}", table_name, e);
    }

    // Deserialize JSON to Rust Structs
    serde_json::from_slice(&buf).unwrap_or_else(|e| {
        eprintln!("❌ JSON Parse Error for '{}': {}", table_name, e);
        vec![]
    })
}

async fn get_doc(Path(_id): Path<String>, State(_state): State<Arc<AppState>>) -> String {
    "Markdown content placeholder...".to_string()
}
