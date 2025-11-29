📄 GraphoDoc: Markdown Knowledge Base Indexer (MVP)

🎯 Core Goal

To transform a directory of Markdown documents (a "Doc Base") into a structured, searchable knowledge graph. This provides a clear, indexable view of document relationships, keywords, and concepts, laying the foundation for future LLM-based RAG (Retrieval-Augmented Generation) and chat applications.

🛠️ Technology Stack

Component

Technology

Role

Language

Rust

High-performance, memory-safe indexing and processing.

Database

lancedb (via Rust API)

Vector-ready, embedded database for persistence and efficient search (future vector embedding search).

Parsing

pulldown-cmark

Converts Markdown content into events for structured data extraction.

NLP/Keywords

stop-words, rust-stemmers

Pre-processing text for generating normalized keywords.

📊 Data Model & Graph Structure

The data is stored across three main tables in LanceDB, representing a basic property graph model:

documents (Nodes)

Fields: id (XXH3 hash of file path), path, title, content (full MD), text (plain text), wiki_links (array), tags (array), keywords (array of stemmed tokens).

nodes (Concepts/Entities)

Represents every unique entity: Documents, Wiki Links (Concepts), Tags, and Keywords.

Fields: id (XXH3 hash of name), name (e.g., "Architecture", "Rust", "Project-A"), kind (Document, Concept, Tag, Keyword), doc_id (Link to the source document if kind is Document).

edges (Relationships)

Defines connections between the nodes.

Fields: source (u64 Node ID), target (u64 Node ID), kind (LinksTo, HasTag, HasKeyword).

⚙️ Processing Pipeline (Indexer)

Scanning: Recursively traverse a user-specified directory, filtering for .md files.

Parsing: For each file:

Read content.

Extract the title from the filename.

Extract wiki_links ([[Target]]) and #tags using Regular Expressions.

Convert Markdown to plain text using pulldown-cmark.

Keyword Extraction:

Tokenize the plain text.

Filter out English stop-words (e.g., "the", "a", "is").

Apply English stemming (e.g., "running" -> "run").

Store unique resulting tokens as keywords.

Indexing:

Persist the processed document data into the documents table.

Iterate over all documents, generating and connecting nodes in the nodes and edges tables based on extracted links, tags, and stemmed keywords.