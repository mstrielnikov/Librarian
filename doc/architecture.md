# GraphoDoc Architecture

## Overview

GraphoDoc transforms a directory of Markdown documents into a searchable knowledge graph optimized for both traditional search and Agentic RAG (Retrieval-Augmented Generation) workflows.

## High-Level Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Markdown      │────▶│   Indexer       │────▶│   LanceDB       │
│   Documents    │     │   Pipeline      │     │   Storage       │
└─────────────────┘     └─────────────────┘     └────────┬────────┘
                                                        │
                       ┌───────────────────────────────┘
                       ▼
              ┌─────────────────┐     ┌─────────────────┐
              │   REST API      │────▶│   Web UI /     │
              │   Server        │     │   Agentic RAG   │
              └─────────────────┘     └─────────────────┘
```

## Core Components

### 1. Indexer Pipeline

The indexer processes Markdown files through multiple stages:

1. **Document Scanning** - Recursively find `.md` files
2. **Content Parsing** - Extract wiki-links `[[Target]]`, tags `#tag`
3. **Text Extraction** - Convert Markdown to plain text
4. **NLP Processing** - Tokenize, stem, remove stop-words
5. **TF-IDF Analysis** - Score keywords by relevance
6. **Graph Building** - Create nodes and edges
7. **Persistence** - Store in LanceDB

### 2. Data Storage (LanceDB)

Three main tables store the knowledge graph:

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `documents` | Source content | id, path, title, content, text, wiki_links, tags, keywords, embedding |
| `nodes` | Graph entities | id (hierarchical), name, kind, doc_id, vector |
| `edges` | Relationships | source, target, kind |

### 3. Query Layer

REST API provides multiple access patterns:

- **Graph Queries** - Full graph, connected nodes, multi-hop traversal
- **Search Queries** - Full-text search across content/keywords/tags
- **Hybrid** - Combine graph structure with semantic search

## Design Principles

### Hierarchical Node Addressing

Node IDs encode structural information for efficient traversal:

```
Document:     U32(0), U32(1), U32(2), ...
Concept:      U64(doc_id << 32 | index)
Tag/Keyword:  U64(hash_with_prefix)
```

Benefits:
- Related nodes have adjacent IDs
- Database can efficiently range-query subgraphs
- ID encodes parent-child relationships

### Vector-Ready Storage

The schema includes embedding fields:

- `Doc.embedding: Option<Vec<f32>>` - Document vectors
- `Node.vector: Option<Vec<f32>>` - Entity vectors

This enables:
- Semantic similarity search
- Hybrid retrieval (keyword + vector)
- Re-ranking by relevance

### TF-IDF Keyword Scoring

Keywords are scored using Term Frequency-Inverse Document Frequency:

```
TF-IDF = (term_count / doc_length) * (ln(N / doc_frequency) + 1)
```

Only keywords with score > 0.5 are included in the graph, reducing noise while capturing important concepts.

## Agentic RAG Integration

The architecture supports future Agentic RAG workflows:

### Retrieval Patterns

1. **Keyword Search** → Get relevant documents
2. **Graph Traversal** → Expand context via connections
3. **Vector Search** → Semantic similarity (future)
4. **Hybrid** → Combine multiple signals

### Context Assembly

```
Query: "What is Grover's algorithm?"

1. Search keywords → [Document: Grover's algorithm]
2. Get connected → [Concepts: quantum, oracle, amplitude amplification]
3. Get neighbors → [Keywords: speedup, search, quadratic]
4. Assemble context → Full document + related concepts + keywords
```

### Future Enhancements

- Add embedding generation (OpenAI, local models)
- Implement vector similarity search in LanceDB
- Add re-ranking using graph structure
- Support for document chunking with parent links

## Technology Stack

| Component | Technology | Role |
|-----------|------------|------|
| Language | Rust | High-performance processing |
| Database | LanceDB | Vector-ready storage |
| Markdown | pulldown-cmark | Parsing |
| NLP | rust-stemmers, stop-words | Text processing |
| Web Server | axum | REST API |
| Frontend | egui (WASM) | Visualization |

## File Structure

```
crates/
├── core/           # Data structures and graph logic
│   └── src/
│       ├── lib.rs  # NodeId, Node, Edge, Doc types
│       └── graph.rs # TF-IDF + graph building
├── cli/           # Indexer and server
│   └── src/
│       ├── indexer.rs # Document processing
│       └── server.rs  # REST API endpoints
└── web/           # WASM frontend
    └── src/
        └── lib.rs  # egui visualization
```
