# MollyDB

<<<<<<< Updated upstream
MollyDB is built to be a high-performance, in-memory SQL database designed for complete compatibility with SQLite.

## Features Implemented To-Date

- Basic CRUD operations (CREATE TABLE, SELECT, UPDATE, DELETE).
- A SQL tokenizer and parser for generating AST representations of CRUD operations.
- A lightweight in-memory representation of a SQLite database.
- A CLI functioning as a UI for the database.

## Stats  

=======
>>>>>>> Stashed changes
[![Wakatime](https://wakatime.com/badge/user/9641004b-568b-4c27-99c5-a34ace36b886/project/2668a03d-d729-4e59-8fc8-bafe3d194ee1.svg)](https://wakatime.com/badge/user/9641004b-568b-4c27-99c5-a34ace36b886/project/2668a03d-d729-4e59-8fc8-bafe3d194ee1)
![GitHub last commit](https://img.shields.io/github/last-commit/FletcherDares/mollydb)
![GitHub stars](https://img.shields.io/github/stars/FletcherDares/mollydb?style=social)

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

MollyDB is currently in early development with many of the listed features yet to be implemented.

<<<<<<< Updated upstream
- A CLI has been developed which can be used to play around with the implemented features.
  - Installing Rust and running `cargo build && cargo run` within the repository will start the CLI.
- Contributions and ideas are welcome! Current progress is tracked using the issues tab on github.
=======
### Implemented Features

- SQL Tokenizer & Parser
- Full CRUD operations (CREATE, SELECT, UPDATE, DELETE, ALTER)
- Transaction statements (BEGIN, COMMIT, ROLLBACK)
- Basic table operations

### Under Current Developement

- Full functionality for table joins
- Comprehensive parity testing with SQLite
- Performance benchmarking suite

### Roadmap

- Row-based caching and eviction policies
- Snapshotting and persistence
- Concurrent read operations
>>>>>>> Stashed changes

## Core Design Philosophy

1. MollyDB is built to be significantly more performant than traditional Disk-based SQL databases (MySQL, Postgres, SQLite) and should be similar in performance to in-memory cache stores (Memcached, Redis).
2. MollyDB is meant to live in-memory, disk I/O should only be performed at direct request of the user (i.e. snapshotting).
3. MollyDB maintain complete parity with SQLite, queries should have full compatibility and produce the same results.
4. MollyDB maintains a test coverage of >75%.

## Features

<<<<<<< Updated upstream
1. **Row Based Caching**
    - MollyDBs primary use case is to function as a modified query cache. Unlike traditional query caches, MollyDB caches and evicts individual rows in it's In-Memory database, the advantage of row based caching is the idea that many queries exist as subsets of one or more other queries.
        - Imagine you have a query for a products preview page which performs the following search upon loading `SELECT id, name, price ORDER BY created_at desc LIMIT 10;` and hovering over the first row of product runs the following query `SELECT image, colors WHERE id IN (123, 124, 125);`.
        - A user then clicks on a product taking them to a product view page running this query: `SELECT name, image, price, colors WHERE id = 123;`
        - In a traditional query cache these would be stored as three seperate objects despite the third query's results being a subset of the first two.
    - With MollyDB, the results of the first and second query would be cached therefore making the third query a cache hit.
    - MollyDB increase the number of cache hits by increasing the total amount of data able to be cached within a memory constraint by completely preventing the duplication of database rows.
    - MollyDB also improves the number of cache hits by increasing the total number of possible queries that can be hits which leads to rarer queries being cache hits with the same memory requirements.
2. **Snapshotting**
    - Complete parity with SQLite allows you to load schemas and data from a SQLite database to use as a datasource specifying snapshot backup intervals and fetching intervals.
3. **Concurrent Reads**
    - The entire database is built to be atomic and thread safe allowing for concurrent reads to the database. Forking is also supported at the risk of loss of data synchronization between forks.
=======
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

Code contributions must run through the formatter before being merged. Run the formatter with `cargo fmt --all`.
>>>>>>> Stashed changes
