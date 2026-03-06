use serde::{Deserialize, Serialize};
use std::fmt;

pub mod graph;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeId {
    U32(u32),
    U64(u64),
    U128(u128),
    Chunked(Vec<u32>),
}

impl Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_hex_string())
    }
}

impl<'de> Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.contains('_') {
            let chunks: Vec<u32> = s
                .split('_')
                .filter_map(|x| u32::from_str_radix(x, 16).ok())
                .collect();
            Ok(NodeId::Chunked(chunks))
        } else if s.len() <= 8 {
            Ok(NodeId::U32(u32::from_str_radix(&s, 16).unwrap_or(0)))
        } else if s.len() <= 16 {
            Ok(NodeId::U64(u64::from_str_radix(&s, 16).unwrap_or(0)))
        } else {
            Ok(NodeId::U128(u128::from_str_radix(&s, 16).unwrap_or(0)))
        }
    }
}

impl NodeId {
    pub fn new_document(doc_index: u32) -> Self {
        NodeId::U32(doc_index)
    }

    pub fn new_concept(doc_id: &NodeId, local_index: u32) -> Self {
        match doc_id {
            NodeId::U32(d) => {
                let shifted = (*d as u64) << 32;
                NodeId::U64(shifted | (local_index as u64))
            }
            NodeId::U64(d) => {
                let shifted = (*d as u128) << 32;
                NodeId::U128(shifted | (local_index as u128))
            }
            NodeId::U128(d) => NodeId::Chunked(vec![
                ((*d >> 96) & 0xFFFFFFFF) as u32,
                ((*d >> 64) & 0xFFFFFFFF) as u32,
                ((*d >> 32) & 0xFFFFFFFF) as u32,
                (*d & 0xFFFFFFFF) as u32,
                local_index,
            ]),
            NodeId::Chunked(c) => {
                let mut new_chunk = c.clone();
                new_chunk.push(local_index);
                NodeId::Chunked(new_chunk)
            }
        }
    }

    pub fn new_tag(name: &str) -> Self {
        Self::from_string_prefixed("tag", name)
    }

    pub fn new_keyword(name: &str) -> Self {
        Self::from_string_prefixed("kw", name)
    }

    fn from_string_prefixed(prefix: &str, name: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}:{}", prefix, name.to_lowercase()).hash(&mut hasher);
        let hash = hasher.finish();

        if hash >> 32 == 0 {
            NodeId::U32(hash as u32)
        } else {
            NodeId::U64(hash)
        }
    }

    pub fn to_hex_string(&self) -> String {
        match self {
            NodeId::U32(n) => format!("{:08x}", n),
            NodeId::U64(n) => format!("{:016x}", n),
            NodeId::U128(n) => format!("{:032x}", n),
            NodeId::Chunked(c) => c
                .iter()
                .map(|x| format!("{:08x}", x))
                .collect::<Vec<_>>()
                .join("_"),
        }
    }

    pub fn kind(&self) -> NodeKind {
        match self {
            NodeId::U32(n) => NodeKind::from_id(*n),
            NodeId::U64(n) => NodeKind::from_id(*n as u32),
            NodeId::U128(n) => NodeKind::from_id(*n as u32),
            NodeId::Chunked(c) => c
                .first()
                .map(|n| NodeKind::from_id(*n))
                .unwrap_or(NodeKind::Document),
        }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Document = 0b00,
    Concept = 0b01,
    Tag = 0b10,
    Keyword = 0b11,
}

impl NodeKind {
    fn from_id(id: u32) -> Self {
        match id & 0xC0000000 {
            0b00_0000000000000000000000000000 => NodeKind::Document,
            0b01_0000000000000000000000000000 => NodeKind::Concept,
            0b10_0000000000000000000000000000 => NodeKind::Tag,
            _ => NodeKind::Keyword,
        }
    }
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeKind::Document => write!(f, "Document"),
            NodeKind::Concept => write!(f, "Concept"),
            NodeKind::Tag => write!(f, "Tag"),
            NodeKind::Keyword => write!(f, "Keyword"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub doc_id: Option<NodeId>,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Doc {
    pub id: NodeId,
    pub path: String,
    pub title: String,
    pub content: String,
    pub text: String,
    pub wiki_links: Vec<String>,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub doc_id: NodeId,
    pub title: String,
    pub path: String,
    pub score: f32,
    pub snippet: String,
}
