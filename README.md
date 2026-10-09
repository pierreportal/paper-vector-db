# PaperVector DB

A toy vector database written in Rust for local, semantic search over your
documents. It extracts text, splits it into meaningful chunks, turns each chunk
into an embedding, and lets you search it by meaning rather than keywords — all
from a small interactive CLI, with no external services required.

## Features

### Text processing
- **Multi-format ingestion** — reads plain text, Markdown, PDF, and several
  source-code formats (`.txt`, `.md`, `.pdf`, `.rs`, `.ts`, `.jsx`).
- **PDF text extraction** via `pdf-extract`.
- **Structure-aware chunking**:
  - Markdown is split on headings (`#`, `##`, ...), so each section becomes its
    own chunk without splitting a topic across boundaries.
  - Other formats are split on blank lines and greedily packed into chunks up to
    a maximum size, with a minimum size to avoid trivially small chunks.
- **Configurable chunk limits** (`CHUNKER_MIN_CHARS`, `CHUNKER_MAX_CHARS`) so
  chunk sizes can be tuned for retrieval quality.
- **Quote-aware tokenizer** for the CLI, so search queries containing spaces can
  be passed as a single quoted argument.

### Embedding
- **Local embeddings** via [`fastembed`](https://crates.io/crates/fastembed)
  using the `AllMiniLML6V2` model — runs entirely on your machine, no API keys.
- **384-dimensional** vectors, one per chunk.
- **Batch-aware embedding API** (`BatchSize::None | Default | Custom`) for
  tuning throughput.
- Embeddings are generated at both **index time** (documents) and **query time**
  (search), so queries and documents live in the same vector space.

### Vector search
- **Cosine similarity** search across all stored chunk embeddings, returning the
  top-K results.
- **`lucky`** command for a single best match and **`search`** for the top-K
  (default 5, configurable via `DEFAULT_K_SEARCH_RESULTS`).
- **Persistent collection** stored in a single bincode-serialized `.db` file,
  loaded automatically on startup.
- **PCA visualization** — projects all embeddings down to 2D so you can eyeball
  how semantically similar documents cluster together.

## How it works

```
file ──▶ text extraction ──▶ chunking ──▶ embedding ──▶ collection (.db)
                                                            │
query ──▶ embedding ──▶ cosine similarity ──▶ top-K results ◀┘
```

1. **Extract** — PDF/text/source files are read to a string.
2. **Chunk** — text is split into sections/paragraph groups within configured
   size bounds.
3. **Embed** — every chunk is embedded into a 384-dim vector.
4. **Store** — the collection is saved to disk with `bincode`.
5. **Search** — the query is embedded and compared against every stored vector
   using cosine similarity.

## Getting started

### Prerequisites
- Rust (edition 2024 toolchain)
- An internet connection on first run so `fastembed` can download the embedding
  model (cached in `.fastembed_cache/`).

### Build & run

```bash
cargo run
```

You'll land in an interactive prompt:

```
paper-db ~>
```

## Commands

| Command | Description |
| --- | --- |
| `embed <path>` | Extract, chunk, and embed a single file. |
| `index <path>` | Recursively walk a directory, embedding every supported file. |
| `search <query>` | Return the top-K (default 5) most similar chunks. |
| `lucky <query>` | Return the single best matching chunk. |
| `all` | List every embedded document and its chunk index. |
| `map` | Project all embeddings to 2D with PCA and print coordinates. |
| `delete <path>` | Remove all chunks belonging to a file path. |
| `deleteById <id>` | Remove a single chunk by its UUID. |
| `quit` / `exit` | Leave the CLI. |

Queries with spaces can be wrapped in quotes:

```
paper-db ~> search "how do transformers attend to tokens"
```

## Configuration

Tunable constants live in `src/config.rs`:

| Constant | Default | Purpose |
| --- | --- | --- |
| `DEFAULT_K_SEARCH_RESULTS` | `5` | Number of results returned by `search`. |
| `DEFAULT_EMBEDDING_DIMENSION` | `384` | Embedding size (`AllMiniLML6V2`). |
| `DEFAULT_EMBEDDNG_BATCH_SIZE` | `256` | Default embedding batch size. |
| `CHUNKER_MAX_CHARS` | `4000` | Maximum characters per chunk. |
| `CHUNKER_MIN_CHARS` | `100` | Minimum characters per chunk. |
| `DEFAULT_COLLECTION_NAME` | `my_collection` | Collection name and storage file stem. |

## Project layout

```
src/
├── main.rs                  # REPL loop and command dispatch
├── config.rs                # Tunable constants
├── cli/                     # clap command definitions + parser
├── text_processing/
│   ├── chunker.rs           # Markdown/paragraph chunking
│   ├── embedder.rs          # fastembed wrapper
│   └── tokenize.rs          # quote-aware CLI tokenizer
├── db/
│   ├── collection.rs        # In-memory collection + cosine search
│   └── storage.rs           # bincode persistence
├── types/document.rs        # Document / DocumentInsert types
├── utils/                   # File validation + text extraction
└── visualization/pca.rs     # PCA projection to 2D
```

## Development

```bash
cargo test      # unit tests for chunking, tokenizing, search, parsing, PCA
cargo clippy    # lints
```

## Notes & limitations

This is a **toy** database, built for learning and experimentation:

- Search is a brute-force scan over all vectors (no ANN index yet).
- No metadata filtering; results carry only path, chunk index, and score.
- Storage is a single serialized file, not a transactional database.
- Embedding is sequential per chunk; parallelism is a possible future
  improvement.
