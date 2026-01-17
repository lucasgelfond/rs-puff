# puff-rs Agent Guide

Build a feature-complete, tested, rigorously typed Rust client for Turbopuffer.

## Goal

Create a Rust SDK that mirrors the official Turbopuffer clients (Go, TypeScript, Python, etc.) with idiomatic Rust patterns. Reference `other-clients/` for structure and test coverage.

## Structure

Use small, focused files with module index exports:

```
src/
  lib.rs              # Re-exports public API
  client.rs           # Client struct, configuration
  namespace.rs        # Namespace operations
  error.rs            # Error types
  types/
    mod.rs            # Re-exports
    id.rs             # Document ID (u64 | String)
    distance_metric.rs
    vector.rs
    row.rs
    filter/
      mod.rs          # Filter trait + re-exports
      eq.rs
      comparison.rs   # Lt, Lte, Gt, Gte
      contains.rs
      glob.rs
      logical.rs      # And, Or, Not
    rank_by/
      mod.rs          # RankBy trait + re-exports
      vector.rs       # ANN, kNN
      bm25.rs
      attribute.rs
    aggregate.rs
    schema.rs
    write.rs          # WriteParams, WriteResponse
    query.rs          # QueryParams, QueryResponse
```

## API Mapping

Match the official clients:

| Go                    | Rust                        |
|-----------------------|-----------------------------|
| `NewClient(opts...)`  | `Client::new(api_key)`      |
| `client.Namespace(n)` | `client.namespace(n)`       |
| `ns.Write(ctx, p)`    | `ns.write(&params).await`   |
| `ns.Query(ctx, p)`    | `ns.query(&params).await`   |
| `ns.DeleteAll(ctx,p)` | `ns.delete_all().await`     |
| `ns.Metadata(ctx,p)`  | `ns.metadata().await`       |

## Types

### Filter (sealed trait pattern)

Use an enum or sealed trait for type-safe filters:

```rust
pub enum Filter {
    Eq { attr: String, value: Value },
    In { attr: String, values: Vec<Value> },
    And(Vec<Filter>),
    Or(Vec<Filter>),
    // etc.
}
```

Provide builder functions: `Filter::eq("status", "active")`

### RankBy

```rust
pub enum RankBy {
    Vector { attr: String, query: Vec<f32> },
    Bm25 { attr: String, query: String },
    Attribute { attr: String, order: Order },
}
```

### Id

```rust
pub enum Id {
    Uint(u64),
    String(String),
}
```

## Endpoints

Implement all namespace operations:

- `POST /v2/namespaces/:ns` - Write (upsert, patch, delete)
- `POST /v2/namespaces/:ns/query` - Query
- `DELETE /v2/namespaces/:ns` - Delete namespace
- `GET /v1/namespaces/:ns/metadata` - Metadata
- `GET /v1/namespaces/:ns/schema` - Schema
- `POST /v1/namespaces/:ns/schema` - Update schema
- `GET /v1/namespaces/:ns/hint_cache_warm` - Warm cache
- `GET /v1/namespaces` - List namespaces

## Tests

Port tests from other clients. Each file should have corresponding tests:

```
tests/
  write_test.rs
  query_test.rs
  filter_test.rs
```

Use a mock server or the actual API with test namespaces (clean up after).

## Style

- Minimal code, no unnecessary abstractions
- Only add comments where logic isn't obvious
- Use standard Rust patterns (Result, Option, async/await)
- Derive common traits: Debug, Clone, Serialize, Deserialize
- Split logic into small files; use `mod.rs` for re-exports

## Dependencies

Keep minimal:
- `serde`, `serde_json` - serialization
- `reqwest` - HTTP client
- `thiserror` - error handling
- `tokio` - async runtime (dev dependency for tests)

## Priority

1. Client + Namespace scaffolding
2. Write operation (upsert_rows)
3. Query operation (basic)
4. Filter types
5. RankBy types
6. Remaining operations
7. Tests
8. Documentation
