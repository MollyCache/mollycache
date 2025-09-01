# MollyDB
MollyDB is built to be a high-performance, in-memory SQL database designed for complete compatibility with SQLite. This is built for applications that require speed over persistence.

## Current Implementation Status
MollyDB is currently in early development with many of the listed features yet to be implemented.
- A CLI has been developed which can be used to play around with the implemented features.
- Contributions and ideas are welcome! Feel free to open pull requests with improvements. Test coverage requirements for contributed files is mandatory.

## Core Design Philosophy:
1. MollyDB is built to be significantly more performant than traditional SQL databases.
2. MollyDB is meant to live in-memory, disk I/O should only be performed at direct request of the user.
3. MollyDB is replacement for SQLite, queries should have complete parity and compatibility.
4. MollyDB maintains a test coverage of >80%.

## Features:
1. Row caching: The core feature of MollyDB is the idea of row caching. Similar to a query cache, once a query to a database is performed the result rows are cached into a MollyDB instance. Once a cache is warm, subsequent queries can be specified to be run against MollyDB instead of the database. However because MollyDB stores results as rows in a database you can perform new queries against the cache instead of fetching exact queries. This sacrifices accuracy / completeness of the results in exchange for better performance.

This works off the idea that there exists queries that are complete subsets of one or more other queries. By storing the results in a row cache instead of query cache you can leverage this to increase cache hits.

2. Snapshotting: Complete parity with SQLite allows you to load schemas and data from a SQLite database to use as a datasource specifying snapshot backup intervals and fetching intervals. 

3. Concurrent Reads: The entire database is built to be atomic and thread safe allowing for concurrent reads to the database. Forking is also supported at the risk of loss of data synchronization between forks.