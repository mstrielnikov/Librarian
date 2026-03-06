# Knowledge Graph: Storage & Addressing

## Graph Representation

The knowledge graph follows a property graph model with three core elements:

### Nodes

Represent entities in the document corpus:

```rust
struct Node {
    id: NodeId,        // Hierarchical address
    name: String,      // Display name
    kind: NodeKind,    // Document, Concept, Tag, Keyword
    doc_id: Option<NodeId>,  // Parent document (for concepts)
    vector: Option<Vec<f32>>, // Embedding for semantic search
}
```

### Edges

Represent relationships between entities:

```rust
struct Edge {
    source: NodeId,
    target: NodeId,
    kind: String,  // LinksTo, HasTag, HasKeyword
}
```

### Node Kinds

| Kind | Description | Example |
|------|-------------|---------|
| Document | Source markdown file | `Grover's algorithm.md` |
| Concept | Wiki-linked entity | `[[Quantum Computing]]` |
| Tag | Hashtag reference | `#algorithm` |
| Keyword | TF-IDF significant term | `search`, `oracle` |

## Hierarchical Node Addressing

### Motivation

Traditional systems use hash-based IDs (e.g., XXH3) which:
- Provide uniform distribution
- But lose structural information
- Create poor locality for related entities

### Solution: Hierarchical IDs

We encode the document structure into the ID itself:

```rust
enum NodeId {
    // Sequential IDs for documents (up to ~4 billion)
    U32(u32),
    
    // Concept from document: (doc_id << 32) | local_index
    U64(u64),
    
    // Deep hierarchies
    U128(u128),
    
    // Very deep trees
    Chunked(Vec<u32>),
}
```

### ID Allocation Strategy

```
Document 0 ─┬─ Concept 0  → U64(0 << 32 | 0) = 0x00000000_00000000
           ├─ Concept 1  → U64(0 << 32 | 1) = 0x00000000_00000001
           ├─ Tag: quantum    → U64(hash("tag:quantum"))
           └─ Keyword: oracle  → U64(hash("kw:oracle"))

Document 1 ─┬─ Concept 0  → U64(1 << 32 | 0) = 0x00000001_00000000
           └─ ...
```

### Benefits

1. **Locality**: Related nodes have similar IDs
2. **Efficient Queries**: Range queries can fetch subgraphs
3. **Parent Encoding**: ID encodes parent document
4. **Memory**: u32/u64 instead of 16-byte strings

## Vector Storage

### Embedding Fields

```rust
struct Doc {
    id: NodeId,
    // ... other fields
    embedding: Option<Vec<f32>>,  // Document embedding
}

struct Node {
    id: NodeId,
    // ... other fields
    vector: Option<Vec<f32>>,    // Entity embedding
}
```

### Use Cases

1. **Semantic Search**: Find similar documents by vector distance
2. **Entity Matching**: Identify related concepts via embeddings
3. **Re-ranking**: Use similarity scores to order results

### Future Integration

```rust
// Pseudo-code for vector search
let query_embedding = embed("quantum algorithms");
let results = table
    .nearest_to(query_embedding)
    .filter("kind = 'Document'")
    .limit(10)
    .execute();
```

## Graph Storage in LanceDB

### Schema

```rust
// Documents table
Table "documents" {
    id: U32,              // Document index
    path: String,         // File path
    title: String,        // Document title
    content: String,      // Raw markdown
    text: String,         // Plain text
    wiki_links: List<String>,
    tags: List<String>,
    keywords: List<String>,
    embedding: FixedSizeList<Float32, 384>,  // Optional
}

// Nodes table  
Table "nodes" {
    id: U64,             // Hierarchical ID
    name: String,
    kind: String,
    doc_id: U32,         // Parent document (nullable)
    vector: FixedSizeList<Float32, 384>,     // Optional
}

// Edges table
Table "edges" {
    source: U64,
    target: U64,
    kind: String,
}
```

### Indexing Strategy

1. **Primary Index**: Node ID (hierarchical)
2. **Secondary Index**: Node kind, document ID
3. **Full-Text Index**: Document text, keywords
4. **Vector Index** (future): IVF-PQ on embeddings

## Graph Traversal

### Single Hop

```rust
fn get_connected(node_id: NodeId) -> GraphData {
    edges.filter(|e| e.source == node_id || e.target == node_id)
        .include_nodes()
}
```

### Multi-Hop Traversal

```rust
fn traverse(start: NodeId, hops: usize) -> GraphData {
    let mut visited = {start};
    let mut queue = {start};
    
    for _ in 0..hops {
        let neighbors = edges
            .filter(|e| queue.contains(e.source))
            .map(|e| e.target)
            .filter(|n| !visited.contains(n));
        
        visited.extend(neighbors);
        queue = neighbors;
    }
    
    return GraphData {
        nodes: nodes.filter(|n| visited.contains(n.id)),
        edges: edges.filter(|e| visited.contains_all([e.source, e.target])),
    };
}
```

## Agentic RAG Patterns

### Pattern 1: Query Expansion

```
User Query: "quantum search"
         │
         ▼
   Keyword Search
         │
         ▼
   Find Related Concepts ──────────┐
         │                         │
         ▼                         ▼
   Graph Traversal         Vector Similarity
         │                         │
         └───────────┬─────────────┘
                     ▼
              Context Assembly
```

### Pattern 2: Multi-Document Reasoning

```
Document A (quantum)
    │
    ├── LinksTo → Concept: Oracle
    │                  │
    │                  └── HasKeyword: superposition
    │
    └── LinksTo → Concept: Amplitude Amplification
                     │
                     └── HasKeyword: quadratic speedup
```

### Pattern 3: Hierarchical Retrieval

```
Level 1: Document (Grover's algorithm.md)
    │
    Level 2: Concepts (oracle, amplitude amplification)
        │
        Level 3: Keywords (quadratic, search, database)
```
