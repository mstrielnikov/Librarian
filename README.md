# GraphoDoc: Markdown Knowledge Base with Graph + Vector Search

Transforms a directory of Markdown documents into a searchable knowledge graph with hybrid (graph + text) search capabilities, optimized for Agentic RAG applications.

## Core Goal

Build a knowledge graph from Markdown documents with:
- **Graph-based relationships** (wiki-links, tags, keywords)
- **TF-IDF keyword analysis** for relevance scoring
- **Vector-ready storage** (LanceDB) for semantic search
- **REST API** for agentic RAG integration

## Technology Stack

| Component | Technology | Role |
|-----------|------------|------|
| Language | Rust | High-performance indexing and processing |
| Database | LanceDB | Vector-ready, embedded DB for persistence and search |
| Markdown | pulldown-cmark | Parse Markdown to plain text |
| NLP | stop-words, rust-stemmers | Text preprocessing + TF-IDF keyword analysis |
| Web | axum + egui (WASM) | REST API + Web UI |

### Prerequisites
- `trunk` - WASM build tool
- `protoc` - Protocol buffer compiler
- Rust toolchain

## Quick Start

```bash
# Build
cargo build --release

# Index a markdown directory
cargo run --release -- --dir ./docs --rebuild

# Run web server (port 3000)
cargo run --release -- --serve --port 3000
```

## Data Model

### Hierarchical Node IDs

The system uses a hierarchical ID scheme for better indexing and graph traversal:

- **Documents**: Sequential `U32` (0, 1, 2, ...)
- **Concepts**: `U64` = `(doc_id << 32) | concept_index` - encodes parent document
- **Tags/Keywords**: Hash-based with kind prefix (deduplicated globally)

```rust
enum NodeId {
    U32(u32),           // Documents (up to ~4B)
    U64(u64),           // Concepts from U32 docs
    U128(u128),         // Deep concepts
    Chunked(Vec<u32>),  // Very deep hierarchies
}
```

### Tables (LanceDB)

**documents** - Source content with embeddings
- `id` - Sequential document index (U32)
- `path`, `title` - File metadata
- `content` - Raw Markdown
- `text` - Plain text
- `wiki_links`, `tags` - Extracted relationships
- `keywords` - Stemmed tokens
- `embedding` - Vector (for future semantic search)

**nodes** - Graph entities
- `id` - Hierarchical NodeId (see above)
- `name` - Entity name
- `kind` - Document, Concept, Tag, Keyword
- `doc_id` - Source document (for concepts)
- `vector` - Entity embedding

**edges** - Relationships
- `source`, `target` - NodeId references
- `kind` - LinksTo, HasTag, HasKeyword

## Processing Pipeline

1. **Scan** - Recursively find `.md` files
2. **Parse** - Extract wiki-links `[[Target]]`, `#tags`
3. **Extract Text** - Convert Markdown to plain text
4. **Keyword Extraction** - Tokenize → stop-word filter → stem → TF-IDF scoring
5. **Build Graph** - Create nodes/edges from links, tags, keywords
6. **Persist** - Save to LanceDB

## TF-IDF Keyword Analysis

Keywords are scored using TF-IDF:
- **TF**: Term frequency within document
- **IDF**: `ln(N/df) + 1` (inverse document frequency)
- Keywords with score > 0.5 are included in the graph

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/graph` | GET | Full graph (nodes + edges) |
| `/api/doc/:id` | GET | Document content |
| `/api/search?q=<query>&limit=<n>` | GET | Full-text search |
| `/api/connected/:node_id` | GET | Direct neighbors |
| `/api/traverse` | POST | Multi-hop traversal |

### Search API

```bash
# Search documents
curl "http://localhost:3000/api/search?q=rust&limit=5"

# Get connected nodes
curl "http://localhost:3000/api/connected/abc123"

# Multi-hop traversal
curl -X POST http://localhost:3000/api/traverse \
  -H "Content-Type: application/json" \
  -d '{"node_id": "abc123", "hops": 2}'
```

## Agentic RAG Integration

The graph structure supports hybrid search:

1. **Text Search** - `/api/search` for keyword matching
2. **Graph Traversal** - `/api/traverse` for related documents
3. **Connected Context** - `/api/connected/:node_id` for immediate neighbors

Future enhancements:
- Add embeddings to `embedding` field for vector similarity
- Use LanceDB's native vector search
- Implement re-ranking with graph structure

## Project Structure

```
graphodoc/
├── Cargo.toml           # Workspace config
├── crates/
│   ├── core/           # Data structures (Node, Edge, Doc)
│   │   └── src/
│   │       ├── lib.rs  # Core types
│   │       └── graph.rs # Graph building + TF-IDF
│   ├── cli/            # Indexer + Server
│   │   └── src/
│   │       ├── main.rs  # CLI entry
│   │       ├── indexer.rs # Document processing
│   │       └── server.rs  # REST API
│   └── web/            # WASM frontend (egui)
│       └── src/
│           └── lib.rs  # Graph visualization
└── mdkb_data/          # LanceDB storage (created on first run)
```

## Usage Examples

```bash
# Index documents with rebuild
cargo run -- --dir ./math --rebuild

# Start server on custom port
cargo run -- --serve --port 8080

# Index + serve in one command
cargo run -- --dir ./docs --serve
```

## Documentation

See the `doc/` directory for detailed documentation:

- **[architecture.md](doc/architecture.md)** - High-level system design and components
- **[knowledge_graph.md](doc/knowledge_graph.md)** - Graph storage, hierarchical addressing, vector embeddings
- **[graph_search.md](doc/graph_search.md)** - TF-IDF algorithm, keyword extraction, search APIs
