use crate::{Doc, Edge, GraphData, Node, NodeId, NodeKind};
use std::collections::{HashMap, HashSet};

pub fn build_graph_from_docs(docs: &[Doc]) -> GraphData {
    let mut nodes_map: HashMap<NodeId, Node> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();

    let total_docs = docs.len();
    if total_docs == 0 {
        return GraphData {
            nodes: vec![],
            edges: vec![],
        };
    }

    let mut doc_frequency: HashMap<String, usize> = HashMap::new();
    let mut processed_docs: Vec<HashMap<String, usize>> = Vec::new();

    for doc in docs {
        let mut term_counts: HashMap<String, usize> = HashMap::new();
        let unique_terms: HashSet<String> = doc.keywords.iter().cloned().collect();

        for term in unique_terms {
            *term_counts.entry(term.clone()).or_insert(0) += 1;
            *doc_frequency.entry(term).or_insert(0) += 1;
        }
        processed_docs.push(term_counts);
    }

    let idf_scores: HashMap<String, f64> = doc_frequency
        .iter()
        .map(|(term, &df)| {
            let idf = ((total_docs as f64) / (df as f64 + 1.0)).ln() + 1.0;
            (term.clone(), idf)
        })
        .collect();

    let mut keyword_tfidf_scores: HashMap<String, f64> = HashMap::new();

    for (i, doc) in docs.iter().enumerate() {
        if let Some(term_counts) = processed_docs.get(i) {
            let doc_len = doc.keywords.len().max(1) as f64;

            for (term, &count) in term_counts {
                let tf = (count as f64) / doc_len;
                let idf = idf_scores.get(term).copied().unwrap_or(1.0);
                let tfidf = tf * idf;
                *keyword_tfidf_scores.entry(term.clone()).or_insert(0.0) += tfidf;
            }
        }
    }

    let mut sorted_keywords: Vec<(String, f64)> = keyword_tfidf_scores.into_iter().collect();
    sorted_keywords.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let significant_keywords: HashSet<String> = sorted_keywords
        .iter()
        .take_while(|(_, score)| *score > 0.5)
        .map(|(k, _)| k.clone())
        .collect();

    let mut tag_ids: HashMap<String, NodeId> = HashMap::new();
    let mut keyword_ids: HashMap<String, NodeId> = HashMap::new();

    for doc in docs {
        let _doc_id = NodeId::new_document(0);

        nodes_map.insert(
            doc.id.clone(),
            Node {
                id: doc.id.clone(),
                name: doc.title.clone(),
                kind: NodeKind::Document,
                doc_id: Some(doc.id.clone()),
                vector: doc.embedding.clone(),
            },
        );

        for (idx, link) in doc.wiki_links.iter().enumerate() {
            let target_id = NodeId::new_concept(&doc.id, idx as u32);
            edges.push(Edge {
                source: doc.id.clone(),
                target: target_id.clone(),
                kind: "LinksTo".to_string(),
            });
            nodes_map.entry(target_id.clone()).or_insert(Node {
                id: target_id,
                name: link.clone(),
                kind: NodeKind::Concept,
                doc_id: Some(doc.id.clone()),
                vector: None,
            });
        }

        for tag in &doc.tags {
            let tag_id = if let Some(existing) = tag_ids.get(tag) {
                existing.clone()
            } else {
                let id = NodeId::new_tag(tag);
                tag_ids.insert(tag.clone(), id.clone());
                nodes_map.entry(id.clone()).or_insert(Node {
                    id: id.clone(),
                    name: tag.clone(),
                    kind: NodeKind::Tag,
                    doc_id: None,
                    vector: None,
                });
                id
            };
            edges.push(Edge {
                source: doc.id.clone(),
                target: tag_id,
                kind: "HasTag".to_string(),
            });
        }

        for keyword in &doc.keywords {
            if significant_keywords.contains(keyword) {
                let kw_id = if let Some(existing) = keyword_ids.get(keyword) {
                    existing.clone()
                } else {
                    let id = NodeId::new_keyword(keyword);
                    keyword_ids.insert(keyword.clone(), id.clone());
                    nodes_map.entry(id.clone()).or_insert(Node {
                        id: id.clone(),
                        name: keyword.clone(),
                        kind: NodeKind::Keyword,
                        doc_id: None,
                        vector: None,
                    });
                    id
                };
                edges.push(Edge {
                    source: doc.id.clone(),
                    target: kw_id,
                    kind: "HasKeyword".to_string(),
                });
            }
        }
    }

    GraphData {
        nodes: nodes_map.into_values().collect(),
        edges,
    }
}
