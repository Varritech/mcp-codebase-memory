# Building Production MCP Servers: The Codebase Memory Pattern

**Stop making AI agents relearn your codebase every session.** This guide shows you how to build persistent, queryable memory for AI coding agents using the Model Context Protocol (MCP).

## The Problem

You've been there. Your AI agent spends 20 minutes reading through your codebase, you have a great conversation about architecture, and then... session ends. Next time you chat, it's back to square one. "Where's the auth module?" "What database are we using?" The agent has amnesia.

Traditional RAG (Retrieval-Augmented Generation) tries to solve this with vector embeddings. But for code, vectors are wasteful. You don't need semantic similarity—you need to know _exactly_ where the rate limiter lives, what functions call the payment processor, and which files define your GraphQL schema.

## Enter Codebase Memory MCP

The pattern is simple: **index your code into a structured knowledge graph, then query it like a database.** Not vectors. Not full-text search. A graph of symbols, references, and relationships.

### What We're Building

- Single static binary (Rust, zero dependencies)
- Indexes 10,000 files in under 10 minutes
- Queries return in sub-milliseconds
- Supports 158 languages out of the box
- Persistent storage (survives restarts)
- Works with Claude Code, Codex, Cursor, and any MCP-compatible agent

## Step 1: Understanding MCP

MCP (Model Context Protocol) is the USB-C of AI tooling. It's a standard way for AI agents to talk to external tools without custom integrations. Think of it like this:

```
AI Agent ←→ MCP Server ←→ Your Tool
```

The MCP server speaks a standard protocol. Your tool implements the logic. The agent doesn't care what your tool does—it just knows how to ask.

For codebase memory, our MCP server:
1. Receives queries from the agent ("Where's auth?")
2. Searches the knowledge graph
3. Returns precise file locations and code snippets

## Step 2: The Indexing Pipeline

Here's the core insight: **code is already structured**. You don't need ML to understand it. A Rust file has modules, structs, functions, and impl blocks. A Python file has classes, methods, and imports. Parse the AST, extract the symbols, store the relationships.

```rust
// Simplified indexing logic
fn index_file(path: &str) -> Vec<Symbol> {
    let ast = parse(path);  // Tree-sitter for 158 languages
    let mut symbols = vec![];
    
    for node in ast.walk() {
        if node.is_definition() {
            symbols.push(Symbol {
                name: node.name(),
                kind: node.kind(),  // function, class, module, etc.
                path: path,
                line: node.start_line(),
                references: find_references(&node),
            });
        }
    }
    
    symbols
}
```

Tree-sitter is the secret sauce. It's an incremental parsing system that works for basically every language. One parser interface, 158 grammars.

## Step 3: The Knowledge Graph

Don't overthink this. SQLite with FTS5 (full-text search) handles 99% of queries. You need three tables:

```sql
CREATE TABLE symbols (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,  -- function, class, module, constant
    file_path TEXT NOT NULL,
    start_line INTEGER,
    end_line INTEGER,
    signature TEXT,      -- function signature, class definition
    content_hash TEXT    -- for change detection
);

CREATE TABLE references (
    symbol_id INTEGER,
    ref_path TEXT,
    ref_line INTEGER,
    context TEXT         -- 3 lines before/after for context
);

CREATE TABLE file_index (
    path TEXT PRIMARY KEY,
    last_modified INTEGER,
    content_hash TEXT,
    indexed_at INTEGER
);
```

The magic is in the queries. "Where's auth?" becomes:

```sql
SELECT s.*, f.path 
FROM symbols s 
JOIN file_index f ON s.file_path = f.path 
WHERE s.name LIKE '%auth%' 
   OR s.signature LIKE '%auth%'
ORDER BY rank;
```

## Step 4: The MCP Protocol

MCP uses JSON-RPC over stdio. Your server reads JSON from stdin, writes JSON to stdout. Here's the minimal implementation:

```rust
// Read request from stdin
let mut input = String::new();
std::io::stdin().read_line(&mut input);

let request: JsonRpcRequest = serde_json::from_str(&input)?;

// Handle the request
let response = match request.method.as_str() {
    "initialize" => handle_initialize(request.params),
    "tools/call" => handle_tool_call(request.params),
    _ => Err(Error::MethodNotFound),
};

// Write response to stdout
println!("{}", serde_json::to_string(&response)?);
```

For codebase memory, you expose tools like:
- `search_symbols(query, kind)` - Find functions, classes, etc.
- `find_references(symbol_name)` - Where is this used?
- `get_file_context(path, line)` - Show surrounding code

## Step 5: Agent Integration

Once your MCP server is built, agents discover it automatically. For Claude Code:

```json
// ~/.claude/settings.json
{
  "mcpServers": {
    "codebase": {
      "command": "/usr/local/bin/codebase-memory-mcp",
      "args": ["--index", "/path/to/project"]
    }
  }
}
```

Now when you ask "Where's the payment logic?", Claude Code queries your MCP server instead of guessing. The response includes exact file paths and line numbers.

## Performance Optimizations

**Incremental indexing:** Don't reindex unchanged files. Store content hashes, skip files that haven't changed since last index.

**Parallel parsing:** Tree-sitter parsing is CPU-bound. Use rayon to parallelize across all cores:

```rust
let symbols: Vec<Symbol> = files
    .par_iter()
    .flat_map(|path| index_file(path))
    .collect();
```

**Query caching:** Cache frequent queries. "Where's auth?" probably gets asked a lot.

**Compression:** Store code snippets with zstd. 70% size reduction, negligible CPU overhead.

## The Payoff

Before: Agent reads 50 files, burns 50k tokens, takes 3 minutes.
After: Agent queries the graph, gets 3 precise results, burns 500 tokens, takes 8ms.

That's 99% token savings and 200x speedup. At scale, this is the difference between a usable agent and an expensive toy.

## What's Next

This pattern extends beyond code. Documentation, tickets, PRs, incident reports—anything structured can become queryable memory. The agent economy needs tools with memory. Build them.

---

**Fork this repo. Ship your own MCP server. Bold ideas wait for no one.**
