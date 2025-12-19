# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Building and Testing
```bash
# CRITICAL: Always use --no-default-features for development builds
cargo build --no-default-features
cargo test --no-default-features

# Run with debug logging
RUST_LOG=debug cargo run -- index

# Quality checks
cargo check --no-default-features --message-format=short
cargo clippy --no-default-features
cargo fmt --check

# Run a specific test
cargo test test_rust_parser -- --nocapture
```

### Development Workflow
```bash
# Index the current codebase
cargo run -- index

# Search with semantic queries
cargo run -- search "HTTP request handling"

# Watch for changes and auto-reindex
cargo run -- watch

# Start MCP server (stdin mode - default, uses current directory)
cargo run -- mcp

# Start MCP server with explicit path
cargo run -- mcp --path /path/to/project

# Start MCP server in non-git directory
cargo run -- mcp --no-git

# Start MCP server (HTTP mode with debug)
cargo run -- mcp --bind 127.0.0.1:3000 --debug

# View GraphRAG relationships
cargo run -- graphrag search "authentication"
cargo run -- graphrag get-node path/to/file.rs

# AI-powered git operations
cargo run -- commit --all
cargo run -- review
cargo run -- release
```

## Architecture Overview

Octocode is a **Rust-based AI code indexer** that combines semantic search, knowledge graphs (GraphRAG), and MCP server capabilities. It's designed for large codebases with a local-first approach.

### Core Data Flow

```
Indexing: Source Files → Tree-sitter Parser → Symbol Extraction → Embedding Generation → LanceDB Storage
                                                                  ↓
          GraphRAG Analysis ← LLM Descriptions ← Chunk Processing

Search:   Query → Embedding → Vector Similarity Search → Result Ranking → Response

Memory:   Input → Semantic Processing → Vector Storage → Git Context → Persistence
```

### Key Architectural Patterns

#### 1. **Multi-Provider Embedding System** (`src/embedding/`)
- Dynamic model discovery with **no hardcoded dimension mappings**
- Provider auto-detection from model string format (e.g., `voyage-3:model-name`, `fastembed:model-name`)
- Feature-gated compilation: `fastembed` and `huggingface` features control local providers
- Cloud providers: Voyage AI, Jina AI, Google
- All providers implement async `EmbeddingProvider` trait

#### 2. **Language-Agnostic Indexing** (`src/indexer/languages/`)
- Tree-sitter parsers for 10+ languages (Rust, Python, JS/TS, Go, PHP, C++, Ruby, etc.)
- Uniform `LanguageParser` trait for symbol extraction, imports, exports
- Import resolution system (`resolution_utils.rs`) maps import statements to actual file paths across languages
- Each language has its own parser module (e.g., `rust.rs`, `python.rs`, `javascript.rs`)

#### 3. **Intelligent Vector Storage** (`src/store/`)
- **LanceDB** columnar database for fast similarity search
- Growth-aware index optimization in `vector_optimizer.rs`:
  - Automatically recreates indexes as dataset grows
  - Dynamic parameter tuning based on dataset size
  - ~10KB storage per file
- Batch operations via `batch_converter.rs` for efficient bulk inserts
- Metadata tracking in `metadata.rs` for index state persistence

#### 4. **GraphRAG Knowledge Graph** (`src/indexer/graphrag/`)
- **AI-powered relationship extraction** between files using LLMs
- Multi-language import resolver with caching
- Three relationship types:
  - `imports`: Direct dependencies
  - `sibling_module`: Same directory files
  - `parent_module`/`child_module`: Hierarchical structure
- Graph operations: search, get-node, get-relationships, find-path, overview
- LLM integration for file descriptions and relationship discovery

#### 5. **MCP Server Architecture** (`src/mcp/`)
- Dual mode support: **stdin** (default for Claude Desktop) and **HTTP** (for other clients)
- Process management prevents concurrent indexing operations
- Intelligent file watching with debouncing (notify-debouncer-mini)
- Debug mode with enhanced logging and performance monitoring
- MCP Proxy (`mcp_proxy.rs`) enables multi-repository management
- LSP integration (`src/mcp/lsp/`) for language server protocol support

#### 6. **Memory System** (`src/memory/`)
- Persistent storage for insights and code context
- Semantic memory search using same embedding system as main store
- Git integration with automatic commit tagging
- Memory types: code, architecture, bug_fix, optimization, security, testing, documentation
- Uses identical vector optimization strategy as main store

### Module Organization

The codebase follows a clear separation of concerns:

- **`config/`**: TOML-based configuration with template defaults in `config-templates/`
- **`indexer/`**: Core parsing logic, batch processing, differential updates, GraphRAG builder
- **`embedding/`**: Multi-provider embedding system with local and cloud options
- **`store/`**: High-level storage abstractions wrapping LanceDB operations
- **`storage.rs`**: Low-level LanceDB table operations and vector operations
- **`mcp/`**: MCP server implementation with stdin/HTTP modes and LSP support
- **`memory/`**: Semantic memory system with git integration
- **`llm/`**: LLM client using octolib with OpenRouter integration
- **`commands/`**: CLI command implementations (index, search, commit, review, etc.)

### Critical Implementation Details

#### Store Initialization Pattern
All commands that need the store follow this pattern:
```rust
let store = Store::new().await?;
store.initialize_collections().await?;
```

Commands that DON'T need store (handled separately in main.rs):
- `config`, `mcp`, `mcp-proxy`, `commit`, `review`, `release`, `format`, `memory`, `logs`, `models`, `completion`

#### Embedding Provider Selection
Provider is determined by model string prefix:
- `voyage-3:` → Voyage AI
- `jina:` → Jina AI
- `text-embedding-` → Google
- `fastembed:` → FastEmbed (local, requires `fastembed` feature)
- `sentence-transformers:` → HuggingFace (local, requires `huggingface` feature)

#### Import Resolution Strategy
The `resolution_utils.rs` module resolves import statements to file paths:
1. Check direct path with proper extension
2. Check index files (index.js, __init__.py, mod.rs)
3. Handle relative vs absolute imports per language
4. Cache results for performance

#### Vector Index Optimization
The `vector_optimizer.rs` triggers index recreation when:
- Dataset grows significantly (configurable thresholds)
- First-time indexing
- Manual optimization requested
Parameters are tuned based on dataset size for optimal performance.

## Configuration System

- Default templates in `config-templates/` directory
- User config in `~/.config/octocode/` (or platform-specific config dir)
- TOML format with sections for: indexing, embedding, search, graphrag, memory
- EditorConfig integration (`ec4rs`) for code formatting

## API Keys and Environment

Required for full functionality:
```bash
export VOYAGE_API_KEY="..."        # For Voyage AI embeddings (200M free tokens/month)
export OPENROUTER_API_KEY="..."    # For LLM features (commit, review, release)
```

Local-only operation possible with `fastembed` or `huggingface` features (macOS only).

## Testing

- Tests directory exists but is currently empty
- CI/CD via GitHub Actions tests on Ubuntu, Windows, macOS with stable/beta/nightly Rust
- Pre-commit hooks enforce formatting and clippy checks
- Integration tests should be added to `tests/` directory following existing patterns

## Performance Characteristics

- **Indexing**: 100-500 files/second (varies by size/complexity)
- **Search latency**: <100ms for most queries
- **Memory**: ~50MB base + ~1KB per indexed file
- **Storage**: ~10KB per file in LanceDB
- **Scalability**: Tested with 100k+ file codebases

## Adding New Language Support

1. Add tree-sitter grammar to `Cargo.toml`
2. Create parser in `src/indexer/languages/your_lang.rs`
3. Implement `LanguageParser` trait with symbol extraction, imports, exports
4. Register in `src/indexer/languages/mod.rs` `get_parser()` function
5. Add import resolution logic if needed in `resolution_utils.rs`
6. Test with sample files

## Key Dependencies

- **LanceDB** (0.22.3): Vector database with columnar storage
- **Tree-sitter** (0.25.10): Multi-language parsing
- **Tokio** (1.48): Async runtime
- **octolib** (0.2.0): Shared library with LLM client and embedding providers
- **Clap** (4.5): CLI framework
- **reqwest** (0.12): HTTP client for API calls

## Common Pitfalls

1. **Always use `--no-default-features`** during development to avoid Windows doctest conflicts
2. **Symlinks are disabled** in file discovery to prevent infinite recursion
3. **Provider validation fails fast** - invalid models error during provider creation, not usage
4. **MCP server modes are mutually exclusive** - stdin (default) vs HTTP (requires --http flag)
5. **Store initialization must happen before any database operations** - see main.rs pattern
6. **GraphRAG requires LLM API key** - relationship extraction uses OpenRouter
7. **Release profile optimizes for size** (`opt-level = "z"`) - development builds are faster with different settings
