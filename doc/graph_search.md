# Graph Search: Keywords, Indexes & TF-IDF

## Overview

GraphoDoc implements a hybrid search system combining:
1. **Full-text search** - Keyword matching in content
2. **Graph-based ranking** - Using TF-IDF scores
3. **Structural queries** - Graph traversal for context expansion

## Keyword Extraction Pipeline

### Stage 1: Text Extraction

```rust
fn extract_text(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut text = String::new();
    
    for event in parser {
        match event {
            Event::Text(t) | Event::Code(t) => {
                text.push_str(&t);
                text.push(' ');
            }
            _ => {}
        }
    }
    text
}
```

### Stage 2: Tokenization & Filtering

```rust
fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| s.to_lowercase())
        .filter(|s| {
            s.len() > 2 &&                           // Min length
            s.chars().all(char::is_alphanumeric)    // Alphanumeric
        })
        .collect()
}
```

### Stage 3: Stop-word Removal

```rust
lazy_static! {
    static ref STOP_WORDS: HashSet<String> = get(LANGUAGE::English)
        .iter()
        .map(|s| s.to_string())
        .collect();
}

fn remove_stop_words(tokens: Vec<String>) -> Vec<String> {
    tokens.into_iter()
        .filter(|t| !STOP_WORDS.contains(t))
        .collect()
}
```

### Stage 4: Stemming

```rust
lazy_static! {
    static ref STEMMER: Stemmer = Stemmer::create(Algorithm::English);
}

fn stem(tokens: Vec<String>) -> Vec<String> {
    tokens.into_iter()
        .map(|t| STEMMER.stem(&t).to_string())
        .filter(|s| s.len() > 2)  // Remove short stems
        .collect()
}
```

### Stage 5: TF-IDF Scoring

## TF-IDF Algorithm

### Term Frequency (TF)

Measures how often a term appears in a document:

```
TF(term, doc) = term_count_in_doc / total_terms_in_doc
```

### Inverse Document Frequency (IDF)

Measures how rare/important a term is across the corpus:

```
IDF(term) = ln(N / documents_containing_term) + 1

where N = total number of documents
```

### TF-IDF Score

```
TF-IDF(term, doc) = TF(term, doc) × IDF(term)
```

### Implementation

```rust
fn compute_tfidf(docs: &[Doc]) -> HashMap<String, f64> {
    let n = docs.len();
    
    // 1. Compute document frequency
    let mut doc_freq: HashMap<String, usize> = HashMap::new();
    for doc in docs {
        for kw in doc.keywords.iter().collect::<HashSet<_>>() {
            *doc_freq.entry(kw.clone()).or_insert(0) += 1;
        }
    }
    
    // 2. Compute IDF scores
    let idf: HashMap<String, f64> = doc_freq
        .iter()
        .map(|(term, &df)| {
            let score = ((n as f64) / (df as f64 + 1.0)).ln() + 1.0;
            (term.clone(), score)
        })
        .collect();
    
    // 3. Compute TF-IDF
    let mut tfidf: HashMap<String, f64> = HashMap::new();
    for doc in docs {
        let doc_len = doc.keywords.len().max(1) as f64;
        let mut term_counts: HashMap<String, usize> = HashMap::new();
        
        for kw in &doc.keywords {
            *term_counts.entry(kw.clone()).or_insert(0) += 1;
        }
        
        for (term, &count) in term_counts {
            let tf = count as f64 / doc_len;
            let idf_score = idf.get(&term).copied().unwrap_or(1.0);
            *tfidf.entry(term.clone()).or_insert(0.0) += tf * idf_score;
        }
    }
    
    tfidf
}
```

### Keyword Selection

Only keywords with significant TF-IDF scores are included:

```rust
let significant_keywords: HashSet<String> = 
    tfidf_scores
        .iter()
        .sorted_by(|a, b| b.1.partial_cmp(a.1).unwrap())
        .take_while(|(_, score)| *score > 0.5)  // Threshold
        .map(|(k, _)| k.clone())
        .collect();
```

## Indexes in LanceDB

### Full-Text Search Index

LanceDB supports FTS via Tantivy:

```rust
// Create FTS index on text column
table.create_index(&["text"], Index::FTS(FtsIndexBuilder::default()))
    .execute()
    .await?;
```

### Index Strategy

| Column | Index Type | Purpose |
|--------|------------|---------|
| `text` | FTS (BM25) | Full-text search |
| `keywords` | FTS | Keyword matching |
| `id` | BTree | Primary key |
| `kind` | BTree | Filter by node type |
| `doc_id` | BTree | Filter by document |
| `embedding` | IVF-PQ | Vector similarity (future) |

## Search API

### Full-Text Search

```bash
GET /api/search?q=quantum&limit=10
```

```rust
async fn search_docs(query: SearchQuery) -> Vec<SearchResult> {
    let search_term = query.q.to_lowercase();
    
    docs.into_iter()
        .filter(|doc| {
            doc.title.contains(&search_term) ||
            doc.text.contains(&search_term) ||
            doc.keywords.iter().any(|k| k.contains(&search_term)) ||
            doc.tags.iter().any(|t| t.contains(&search_term))
        })
        .map(|doc| SearchResult {
            doc_id: doc.id,
            title: doc.title,
            path: doc.path,
            score: 1.0,
            snippet: doc.text.chars().take(200).collect(),
        })
        .take(query.limit)
        .collect()
}
```

### Connected Nodes

```bash
GET /api/connected/00000001
```

```rust
async fn get_connected(node_id: NodeId) -> GraphData {
    let edges = all_edges
        .iter()
        .filter(|e| e.source == node_id || e.target == node_id);
    
    let node_ids: HashSet<NodeId> = edges
        .flat_map(|e| [e.source, e.target])
        .collect();
    
    GraphData {
        nodes: all_nodes.filter(|n| node_ids.contains(&n.id)),
        edges: edges.collect(),
    }
}
```

### Multi-Hop Traversal

```bash
POST /api/traverse
{"node_id": "00000001", "hops": 2}
```

```rust
async fn traverse(start: NodeId, hops: usize) -> GraphData {
    let mut visited = HashSet::new();
    let mut queue = vec![start];
    visited.insert(start);
    
    for _ in 0..hops {
        let next: Vec<NodeId> = edges
            .iter()
            .filter(|e| queue.contains(&e.source) && !visited.contains(&e.target))
            .map(|e| {
                visited.insert(e.target);
                e.target
            })
            .collect();
        
        if next.is_empty() { break; }
        queue = next;
    }
    
    // Return subgraph
    GraphData { nodes: visited_nodes, edges: visited_edges }
}
```

## Search Ranking

### Score Factors

1. **Text Match**: Keyword appears in title/content
2. **TF-IDF Weight**: Aggregate keyword scores
3. **Graph Proximity**: Distance from query terms
4. **Link Count**: Number of incoming links (PageRank-style)

### Future Enhancements

```rust
// Hybrid scoring
fn rank(doc: &Doc, query: &str) -> f32 {
    let text_score = text_match_score(&doc.text, query);
    let tfidf_score = doc.keywords
        .iter()
        .map(|k| tfidf[k])
        .sum::<f32>();
    let link_score = in_degree[&doc.id] as f32 / max_in_degree;
    
    text_score * 0.4 + tfidf_score * 0.4 + link_score * 0.2
}
```

## Usage Examples

### Basic Search

```bash
# Find documents mentioning "quantum"
curl "http://localhost:3000/api/search?q=quantum"
```

### Graph Expansion

```bash
# Get all nodes connected to document 1
curl "http://localhost:3000/api/connected/00000001"

# Get 2-hop neighborhood
curl -X POST http://localhost:3000/api/traverse \
  -H "Content-Type: application/json" \
  -d '{"node_id": "00000001", "hops": 2}'
```

### Combined Workflow

```rust
// Agentic RAG: Query → Expand → Assemble Context
async fn rag_query(query: &str) -> Context {
    // 1. Find relevant documents
    let docs = search(query, limit=3).await;
    
    // 2. Expand with connected context
    let mut context_nodes = vec![];
    for doc in &docs {
        let connected = get_connected(&doc.id).await;
        context_nodes.extend(connected.nodes);
    }
    
    // 3. Assemble context
    Context {
        documents: docs,
        concepts: context_nodes.filter(NodeKind::Concept),
        keywords: context_nodes.filter(NodeKind::Keyword),
    }
}
```
