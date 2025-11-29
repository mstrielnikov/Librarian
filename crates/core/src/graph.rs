use std::collections::HashMap;
use xxhash_rust::xxh3::xxh3_64;
use crate::{Doc, Node, Edge, GraphData};

pub fn build_graph_from_docs(docs: &[Doc]) -> GraphData {
    let mut nodes_map: HashMap<String, Node> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();

    // --- STEP 1: Keyword Analysis ---
    let mut keyword_counts: HashMap<String, usize> = HashMap::new();
    for doc in docs {
        for keyword in &doc.keywords {
            *keyword_counts.entry(keyword.clone()).or_insert(0) += 1;
        }
    }

    // --- STEP 2: Build Nodes & Edges ---
    for doc in docs {
        // Document Node
        nodes_map.insert(doc.id.clone(), Node {
            id: doc.id.clone(),
            name: doc.title.clone(),
            kind: "Document".to_string(),
            doc_id: Some(doc.id.clone()),
        });

        // Wiki Links
        for link in &doc.wiki_links {
            let target_id = get_or_create_node(link, "Concept", None, &mut nodes_map);
            edges.push(Edge { source: doc.id.clone(), target: target_id, kind: "LinksTo".to_string() });
        }

        // Tags
        for tag in &doc.tags {
            let target_id = get_or_create_node(tag, "Tag", None, &mut nodes_map);
            edges.push(Edge { source: doc.id.clone(), target: target_id, kind: "HasTag".to_string() });
        }

        // Keywords
        for keyword in &doc.keywords {
            if let Some(&count) = keyword_counts.get(keyword) {
                if count >= 2 {
                    let target_id = get_or_create_node(keyword, "Keyword", None, &mut nodes_map);
                    edges.push(Edge { source: doc.id.clone(), target: target_id, kind: "HasKeyword".to_string() });
                }
            }
        }
    }

    GraphData {
        nodes: nodes_map.into_values().collect(),
        edges,
    }
}

fn get_or_create_node(
    name: &str,
    kind: &str,
    doc_id: Option<String>,
    cache: &mut HashMap<String, Node>,
) -> String {
    // Generate Hex String ID
    let hash = xxh3_64(name.as_bytes());
    let id = format!("{:016x}", hash);

    if !cache.contains_key(&id) {
        cache.insert(id.clone(), Node {
            id: id.clone(),
            name: name.to_string(),
            kind: kind.to_string(),
            doc_id,
        });
    }
    id
}
