use serde::{Deserialize, Serialize};

pub mod graph;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub doc_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Doc {
    pub id: String,
    pub path: String,
    pub title: String,
    pub content: String,
    pub text: String,
    pub wiki_links: Vec<String>,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
}
