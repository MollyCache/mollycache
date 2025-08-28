# MollyDB
MollyDB is built to be a high-performance, in-memory SQL database designed for complete compatibility with SQLite. This is built for applications that require speed over persistance.


## Core Design Philopshy:
1. MollyDB is built to be significantly more performant than traditional SQL databases.
2. MollyDB is meant to live in-memory, I/O should only be performed at direct request of the user.
3. MollyDB is replacement for SQLite, queries should have complete parity and compatibility.
4. MollyDB maintains a test coverage of >80%.
