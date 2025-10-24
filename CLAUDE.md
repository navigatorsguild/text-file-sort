# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust crate that implements an external sort algorithm for large text files (CSV, TSV, pg_dump, etc.). The implementation is designed to handle very large files (billions of lines) by using parallel processing across multiple CPU cores with memory usage control.

## Development Commands

### Build
```bash
cargo build
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_parallel_sort

# Run tests with output
cargo test -- --nocapture
```

### Linting
```bash
# Run clippy (integrated as of recent commits)
cargo clippy
cargo clippy -- -D warnings

# Run clippy on all targets (required for comprehensive checks)
cargo clippy --all-targets --all-features -- -D warnings
```

### Benchmarks
```bash
cargo bench
```

### Examples
```bash
cargo run --example sort_text_file
```

## Architecture

### External Sort Algorithm

The crate implements an **external merge sort** designed for files that don't fit in memory:

1. **Chunking Phase** (`chunk_iterator.rs`): Input files are divided into chunks respecting line boundaries. Each chunk is read in configurable sizes (default 10MB).

2. **Parallel Sorting Phase** (`sort.rs`, `sort_command.rs`):
   - Uses a thread pool (via `command-executor` crate) to sort chunks in parallel
   - Each thread maintains thread-local state (`LINE_CAPACITY`, `LINE_RECORDS_CAPACITY`, `SORTED_FILES`, `CONFIG`)
   - Sorted chunks are written to temporary files (`.unmerged` suffix)

3. **Concurrent Merge Phase** (optional, enabled by default):
   - While sorting continues, sorted chunks can be merged concurrently to reduce the number of intermediate files
   - Each thread merges its own sorted chunks

4. **Final Merge Phase** (`internal_merge`):
   - Uses a min-heap (`BinaryHeap<UnmergedChunkFile>`) to efficiently merge all sorted chunks
   - Reads from multiple files simultaneously, always writing the minimum record
   - Removes intermediate files as they're consumed

### Key Components

- **`Sort`** (`sort.rs`): Main public API. Builder pattern for configuration. Manages thread pool, file limits (rlimit), and orchestrates the sort workflow.

- **`LineRecord`** (`line_record.rs`): Represents a single line with extracted keys for comparison. Implements `Ord` based on configured field order (Asc/Desc).

- **`Key`** (`key.rs`): Enum representing different field types (String, Integer, Number). Handles field-specific comparisons and transformations (ignore_blanks, ignore_case, random).

- **`Field`** (`field.rs`): Configuration for a single field in a record. Specifies index (0 = whole line, 1+ = field number), type, and comparison options.

- **`Config`** (`config.rs`): Internal configuration object passed to worker threads. Contains all sorting parameters.

- **`ChunkIterator`** (`chunk_iterator.rs`): Iterator that yields file chunks respecting UTF-8 character boundaries and line endings.

- **`SortedChunkFile`/`UnmergedChunkFile`** (`sorted_chunk_file.rs`, `unmerged_chunk_file.rs`): Wrappers for sorted intermediate files used in the merge phase.

### Thread-Local Storage

The implementation uses thread-local storage extensively to avoid passing shared state:
- `LINE_CAPACITY`: Optimizes string allocation sizes
- `LINE_RECORDS_CAPACITY`: Optimizes vector allocation sizes
- `SORTED_FILES`: Per-thread heap of sorted chunk files
- `CONFIG`: Configuration cloned to each worker thread

### Memory Management

- Optimized for use with Jemalloc allocator (shown in examples, included in dev-dependencies)
- Configurable chunk sizes to control memory usage
- Rlimit management to ensure enough file descriptors for parallel operations
- Thread-local capacities learned during execution to reduce allocations

## Testing Structure

Tests are located in `tests/` directory:
- `test_parallel_sort.rs`: Main sorting tests with multiple tasks
- `test_merge.rs`: Tests for merge functionality
- `test_check.rs`: Tests for sorted file verification
- `test_prefix_sufix.rs`: Tests for prefix/suffix handling
- `common/mod.rs`: Shared test utilities

Test fixtures are in `tests/fixtures/`. Tests use `./target/parallel-results/` for temporary files.

## Important Implementation Notes

- Field indices are 1-based (except index 0 which means "entire line")
- Default field separator is TAB (`\t`)
- Default behavior ignores lines starting with `#`
- The algorithm automatically manages file descriptor limits via rlimit
- Intermediate files use configurable prefix/suffix (default: `part-*.unmerged`)
- Supports custom line endings (default `\n`, CRLF not supported)
- Concurrent merge is enabled by default for better performance

## Public API

The main entry point is `Sort::new(inputs, output)` with builder methods:
- `with_tasks(n)`: Set CPU cores to use (0 = all cores)
- `with_tmp_dir(path)`: Set temporary directory for intermediate files
- `with_chunk_size_bytes(n)` / `with_chunk_size_mb(n)`: Control chunk sizes
- `with_field_separator(char)`: Set delimiter for record parsing
- `add_field(Field)` / `with_fields(Vec<Field>)`: Define sort keys
- `with_order(Order)`: Set Asc or Desc ordering
- `with_prefix_lines(Vec<String>)` / `with_suffix_lines(Vec<String>)`: Add header/footer
- `sort()`: Execute the sort
- `check()`: Verify if files are sorted
- `merge()`: Merge already-sorted files

## Dependencies

Key external dependencies:
- `command-executor`: Thread pool for parallel execution
- `regex`: Pattern matching for ignore rules
- `tempfile`: Temporary file management
- `rlimit`: File descriptor limit control
- `anyhow`: Error handling

## Maintenance Workflow

This repository follows a standardized maintenance workflow documented in `MAINTENANCE_WORKFLOW.md`. Key aspects:

### Zero Warnings Policy
- All code must pass `cargo clippy --all-targets --all-features -- -D warnings` with zero warnings
- Common clippy fixes include:
  - `doc_lazy_continuation`: Indent continuation lines in doc comments
  - `len_zero`: Use `.is_empty()` instead of `.len() == 0`
  - `missing_const_for_thread_local`: Use `const { ... }` for thread_local initializers
  - `unused_io_amount`: Use `.write_all()` instead of `.write()` to ensure all data is written

### Git Workflow
- Always create branch BEFORE making changes
- Branch naming: `maintenance/<description>`
- Commit format: `[MAINTENANCE] #<issue> - <description>`
- Create maintenance issues before starting work

### Publishing
- Repository has GitHub Actions workflow to publish to crates.io on version tags
- Tag format: `v*.*.*` for stable releases, `v*.*.*-*` for pre-releases
- Workflow verifies version, runs tests, and publishes automatically

For full details, see `MAINTENANCE_WORKFLOW.md`.
