# CLAUDE.md

## Working with Claude on MollyCache

This document outlines how Claude (AI assistant) and the development team collaborate on MollyCache to bring this high-performance, in-memory SQL database to market.

## Project Context

**What is MollyCache?**
MollyCache is a from-scratch, in-memory SQL database built in Rust with SQLite compatibility. Unlike traditional query caches that store complete query results, MollyCache implements intelligent row-based caching where individual rows are cached and evicted, enabling superior memory efficiency when queries share overlapping data.

**Current Status:** Early development (v0.1.0)

**Target Market:** Applications requiring high-performance in-memory data access with SQL compatibility, particularly those with overlapping query patterns where traditional query caching falls short.

## Core Development Philosophy

When working on MollyCache, these principles are non-negotiable:

1. **Performance First**: Code must be optimized for in-memory performance comparable to Memcached/Redis, not disk-based databases
2. **In-Memory First**: All data lives in RAM; disk I/O only on explicit user request
3. **SQLite Compatibility**: Complete parity with SQLite queries and results
4. **High Test Coverage**: Maintain >75% test coverage at all times
5. **Zero Dependencies**: Use only Rust standard library - no external crates
6. **Clean, Minimal Code**: Straightforward implementations over clever abstractions

## Development Workflow

### 1. Understanding Tasks
- Check GitHub Issues for context and related discussions
- Review existing code in relevant modules before making changes
- Ask clarifying questions if requirements are ambiguous
- Consider how changes fit into the broader architecture

### 2. Code Standards

**Rust Style:**
- Use Rust Edition 2024
- Follow standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Prefer explicit error handling with `Result<T, String>`
- Keep functions focused and modular
- Use descriptive variable and function names

**Architecture Patterns:**
- **Stack-based state management** for transactions (rows, columns, table names all maintain stacks)
- **Separation of concerns**: Tokenizer → Parser → AST → Database Executor
- **Helper functions** for shared logic (e.g., row filtering, order-by evaluation)
- **Type casting** for SQLite compatibility across Value types

**Error Messages:**
- Clear, actionable error messages
- Include context (table names, column names, etc.)
- Parser errors should include line/column information

### 3. Testing Requirements

**Every code change must include tests:**
- Unit tests embedded in source files with `#[cfg(test)]`
- Integration tests in `/tests` directory for user-facing features
- Use test utilities in `test_utils.rs` for assertions
- Verify edge cases and error conditions
- Ensure tests pass before committing: `cargo test`

**Test Coverage:**
- Run `cargo tarpaulin` to verify coverage remains >75%
- Don't merge code that drops coverage below target

### 4. Pre-Commit Checklist

Before any commit:
```sh
cargo fmt --all        # Format code
cargo test             # All tests must pass
cargo tarpaulin        # Verify coverage >75% (if available)
```

### 5. Commit Messages
- Use clear, descriptive commit messages
- Focus on the "why" not just the "what"
- Follow existing commit style in git log
- One logical change per commit when possible

### 6. Pull Requests
- Reference related GitHub Issues
- Explain architectural decisions in PR description
- Include test plan or verification steps
- Ensure CI passes before requesting review

## How Claude Should Approach Tasks

### Research First
- Read relevant source files before making changes
- Understand existing patterns and architecture
- Check test files to understand expected behavior
- Review recent commits for context

### Propose Solutions
- Present multiple approaches when applicable
- Explain trade-offs (performance, maintainability, complexity)
- Consider impact on existing functionality
- Think about future extensibility

### Implement Incrementally
- Break large features into smaller, testable pieces
- Implement core functionality first, then edge cases
- Keep commits focused and atomic
- Test continuously during development

### Communication Style
- Be concise and technical
- Focus on facts and implementation details
- Avoid unnecessary praise or filler
- Ask specific questions when blocked
- Provide code examples when explaining concepts

## Key Architectural Components

### Database Layer (`src/db/`)
- **Database** (`database.rs`): Central executor, routes SQL statements to appropriate handlers
- **Table** (`table/core/table.rs`): Manages rows and columns with stack-based history
- **Row** (`table/core/row.rs`): Vector of Values with transaction stack support
- **Column** (`table/core/column.rs`): Column definitions with types and constraints
- **Value** (`table/core/value.rs`): Typed data (Integer, Real, Text, Blob, Null) with casting
- **Operations** (`table/operations/`): CRUD implementations (CREATE, INSERT, SELECT, UPDATE, DELETE, DROP, ALTER)
- **Transactions** (`transactions/`): Transaction log, COMMIT, ROLLBACK, SAVEPOINT

### Interpreter Layer (`src/interpreter/`)
- **Tokenizer** (`tokenizer/`): Lexical analysis of SQL strings
- **AST** (`ast/`): Abstract Syntax Tree definitions and parsing
- **Parser** (`ast/parser.rs`): Converts tokens to structured SQL statements

### CLI Layer (`src/cli/`)
- **CLI** (`cli/mod.rs`): Interactive REPL interface

## Common Development Scenarios

### Adding a New SQL Feature
1. Add tokens to `tokenizer/token.rs` if needed
2. Update parser in `interpreter/ast/` to recognize syntax
3. Add statement type to `SqlStatement` enum
4. Implement execution logic in appropriate `operations/` module
5. Add integration tests in `/tests`
6. Update README if user-facing

### Fixing a Bug
1. Write a failing test that reproduces the bug
2. Identify root cause through code review and debugging
3. Implement minimal fix
4. Verify test passes and no regressions
5. Consider edge cases and add additional tests

### Optimizing Performance
1. Benchmark current implementation
2. Profile to identify bottleneck
3. Implement optimization
4. Verify correctness with existing tests
5. Benchmark improvement
6. Document trade-offs in commit message

## Working with Zero Dependencies

Since MollyCache uses only Rust standard library:
- No external crates for parsing, data structures, or algorithms
- Implement solutions from first principles
- Optimize using std library features (HashMap, Vec, etc.)
- Consider performance implications of standard collections

## Future Roadmap Awareness

When making changes, consider these planned features:
- **Row-based caching with eviction policies**: Data structures should support efficient eviction
- **SQLite snapshots**: Architecture should support loading from disk
- **Concurrent reads**: Code should be thread-safe where applicable
- **Multi-table JOINs**: Query execution should be extensible for joins
- **Query optimization**: Consider where indexes or query planning might fit

## Getting Unstuck

If you encounter challenges:
1. Review similar implementations in the codebase
2. Check SQLite documentation for expected behavior
3. Look at test files for usage examples
4. Ask the development team specific technical questions
5. Propose multiple solution approaches with trade-offs

## Goals for Market Launch

To get MollyCache to market, focus areas include:

1. **Core SQL Completeness**: Full SQLite compatibility for common queries
2. **Row-Based Caching**: Implement intelligent caching with eviction policies
3. **Performance Benchmarks**: Demonstrate performance parity with Redis/Memcached
4. **Production Readiness**: Error handling, edge cases, memory safety
5. **Documentation**: Clear usage examples and API documentation
6. **Real-World Testing**: Validate with actual use cases and workloads

## Questions?

When in doubt:
- Check existing code patterns in the repository
- Review test files for examples
- Ask the development team
- Refer to SQLite documentation for compatibility questions

---

**Remember**: MollyCache aims to revolutionize query caching through intelligent row-based storage. Every line of code should serve that mission while maintaining the principles of performance, simplicity, and SQLite compatibility.
