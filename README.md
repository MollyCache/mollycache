# MollyCache

![GitHub last commit](https://img.shields.io/github/last-commit/MollyCache/mollycache)
![GitHub stars](https://img.shields.io/github/stars/MollyCache/mollycache?style=social)
[![Fletcher](https://wakatime.com/badge/user/9641004b-568b-4c27-99c5-a34ace36b886/project/b3cd9856-dee7-41a0-a31c-b3b8b68a0e80.svg)](https://wakatime.com/badge/user/9641004b-568b-4c27-99c5-a34ace36b886/project/b3cd9856-dee7-41a0-a31c-b3b8b68a0e80) 
\+ 86h41m before name change.

MollyCache is a high-performance, in-memory SQL database with row-based caching capabilities, designed for SQLite compatibility.

## Requirements

- [Rust](https://rust-lang.org/tools/install/) v1.88.0 or higher
- Optionally, for testing, the [`tarpaulin`](https://crates.io/crates/cargo-tarpaulin) crate (install with `cargo install tarpaulin`)

## Running

To run the MollyCache interactive CLI:

```sh
cargo run
```

## Testing

To run the entire test suite:

```sh
cargo test
```

To get estimated test coverage (requires the `tarpaulin` crate):

```sh
cargo tarpaulin
```

## Current Implementation Status

MollyCache is currently in **early development**. Many planned features are not yet implemented.

Implemented and planned features are tracked through GitHub Issues.

## Core Design Philosophy

1. **Performance**: MollyCache aims to be significantly more performant than traditional disk-based SQL databases (MySQL, Postgres, SQLite) with performance comparable to in-memory cache stores (Memcached, Redis).
2. **In-Memory First**: MollyCache lives in-memory; disk I/O is only performed at direct request of the user (e.g., snapshotting).
3. **SQLite Compatibility**: MollyCache works towards complete parity with SQLite—queries should have full compatibility and produce the same results.
4. **High Test Coverage**: MollyCache maintains a test coverage goal of >75%.
5. **Zero Dependencies**: MollyCache remains dependency-free, using only Rust standard library functions.

## Planned Features

### Row-Based Caching

MollyCache's primary use case is to function as an intelligent query cache. Unlike traditional query caches that store complete query results, MollyCache caches and evicts individual rows in its in-memory database.

**Why row-based caching?** Many queries exist as subsets of one or more other queries.

#### Example scenario

- Product preview page loads: `SELECT id, name, price FROM products ORDER BY created_at DESC LIMIT 10;`
- User hovers over products: `SELECT image, colors FROM products WHERE id IN (123, 124, 125);`
- User clicks a product: `SELECT name, image, price, colors FROM products WHERE id = 123;`

In a traditional query cache, these would be stored as three separate objects despite the third query's results being a subset of the first two. With MollyCache, the results of the first and second queries would be cached as rows, making the third query a cache hit.

### Snapshotting

Complete parity with SQLite allows you to load schemas and data from SQLite databases to use as a data source, with configurable snapshot backup intervals and fetch intervals.

### Concurrent Reads

The entire database is built to be atomic and thread-safe, allowing for concurrent reads. Forking is also supported at the risk of loss of data synchronization between forks.

## Contributing

Contributions and ideas are welcome! Current progress is tracked using the [issues tab on GitHub](https://github.com/MollyCache/mollycache/issues).

Code contributions must be properly formatted before being merged. Run the formatter with `cargo fmt --all`.
