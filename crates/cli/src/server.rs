use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router, http::StatusCode,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use graphodoc_core::{GraphData, Node, Edge, SearchResult, NodeId};
use futures::TryStreamExt;
use lancedb::query::ExecutableQuery;
use serde::Deserialize;

fn parse_node_id(s: &str) -> NodeId {
    if s.contains('_') {
        let chunks: Vec<u32> = s
            .split('_')
            .filter_map(|x| u32::from_str_radix(x, 16).ok())
            .collect();
        NodeId::Chunked(chunks)
    } else if s.len() <= 8 {
        NodeId::U32(u32::from_str_radix(s, 16).unwrap_or(0))
    } else if s.len() <= 16 {
        NodeId::U64(u64::from_str_radix(s, 16).unwrap_or(0))
    } else {
        NodeId::U128(u128::from_str_radix(s, 16).unwrap_or(0))
    }
}

struct AppState {
    db_path: PathBuf,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct TraverseQuery {
    node_id: String,
    hops: Option<usize>,
}

pub async fn start_server(port: u16, db_path: PathBuf) -> anyhow::Result<()> {
    let state = Arc::new(AppState { db_path });
    let web_dist = PathBuf::from("./crates/web/dist");

    let app = Router::new()
        .route("/api/graph", get(get_graph))
        .route("/api/doc/:id", get(get_doc))
        .route("/api/search", get(search_docs))
        .route("/api/traverse", post(traverse_graph))
        .route("/api/connected/:node_id", get(get_connected_nodes))
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

async fn search_docs(
    Query(query): Query<SearchQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let db = lancedb::connect(state.db_path.to_str().unwrap_or("./mdkb_data"))
        .execute()
        .await
        .map_err(|e| {
            eprintln!("❌ DB Connect Error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let limit = query.limit.unwrap_or(10);
    let search_term = query.q.to_lowercase();

    let docs: Vec<Doc> = fetch_table_safe(&db, "documents").await;
    
    let results: Vec<SearchResult> = docs
        .into_iter()
        .filter(|doc| {
            doc.title.to_lowercase().contains(&search_term) ||
            doc.text.to_lowercase().contains(&search_term) ||
            doc.keywords.iter().any(|k| k.to_lowercase().contains(&search_term)) ||
            doc.tags.iter().any(|t| t.to_lowercase().contains(&search_term))
        })
        .map(|doc| {
            let snippet = doc.text.chars().take(200).collect::<String>();
            SearchResult {
                doc_id: doc.id,
                title: doc.title,
                path: doc.path,
                score: 1.0,
                snippet,
            }
        })
        .take(limit)
        .collect();

    println!("🔍 Search '{}' returned {} results", query.q, results.len());
    Ok(Json(results))
}

#[derive(serde::Deserialize)]
struct Doc {
    id: NodeId,
    title: String,
    path: String,
    text: String,
    keywords: Vec<String>,
    tags: Vec<String>,
}

async fn get_connected_nodes(
    Path(node_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<GraphData>, StatusCode> {
    let db = lancedb::connect(state.db_path.to_str().unwrap_or("./mdkb_data"))
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let nodes: Vec<Node> = fetch_table_safe(&db, "nodes").await;
    let edges: Vec<Edge> = fetch_table_safe(&db, "edges").await;

    let target_id = parse_node_id(&node_id);
    
    let connected_ids: std::collections::HashSet<NodeId> = edges
        .iter()
        .filter(|e| e.source == target_id || e.target == target_id)
        .flat_map(|e| vec![e.source.clone(), e.target.clone()])
        .collect();

    let filtered_nodes: Vec<Node> = nodes
        .into_iter()
        .filter(|n| connected_ids.contains(&n.id))
        .collect();

    let filtered_edges: Vec<Edge> = edges
        .into_iter()
        .filter(|e| e.source == target_id || e.target == target_id)
        .collect();

    Ok(Json(GraphData {
        nodes: filtered_nodes,
        edges: filtered_edges,
    }))
}

async fn traverse_graph(
    State(state): State<Arc<AppState>>,
    Json(params): Json<TraverseQuery>,
) -> Result<Json<GraphData>, StatusCode> {
    let db = lancedb::connect(state.db_path.to_str().unwrap_or("./mdkb_data"))
        .execute()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let nodes: Vec<Node> = fetch_table_safe(&db, "nodes").await;
    let edges: Vec<Edge> = fetch_table_safe(&db, "edges").await;

    let start_id = parse_node_id(&params.node_id);
    let hops = params.hops.unwrap_or(1);
    let mut visited: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
    let mut queue: Vec<NodeId> = vec![start_id.clone()];
    let mut found_edges: Vec<Edge> = Vec::new();

    visited.insert(start_id);

    for _ in 0..hops {
        let mut next_queue: Vec<NodeId> = Vec::new();
        
        for edge in &edges {
            if queue.contains(&edge.source) && !visited.contains(&edge.target) {
                visited.insert(edge.target.clone());
                next_queue.push(edge.target.clone());
                found_edges.push(edge.clone());
            } else if queue.contains(&edge.target) && !visited.contains(&edge.source) {
                visited.insert(edge.source.clone());
                next_queue.push(edge.source.clone());
                found_edges.push(edge.clone());
            }
        }
        
        queue = next_queue;
        if queue.is_empty() {
            break;
        }
    }

    let found_nodes: Vec<Node> = nodes
        .into_iter()
        .filter(|n| visited.contains(&n.id))
        .collect();

    Ok(Json(GraphData {
        nodes: found_nodes,
        edges: found_edges,
    }))
}
