# Fast File Finder

A high-performance file search utility written in Rust that combines filename pattern matching with content searching. It features parallel processing, memory mapping for large files, and context-aware result display.

## Features

- Parallel file processing using Rayon
- Memory mapping for efficient large file handling (>10MB)
- Regex-based filename and content matching
- Configurable context lines around matches
- Colored output for better readability
- Optimized circular buffer for context management

## Installation

```bash
cargo install --path .
```

## Usage

```bash
file-finder [OPTIONS] <ROOT>
```

### Arguments

- `<ROOT>`: Root directory to search

### Options

- `-n, --name <PATTERN>`: Filename regex pattern
- `-c, --content <PATTERN>`: Content regex pattern
- `-C, --context <LINES>`: Context lines around matches (default: 1)
- `-j, --jobs <NUM>`: Number of parallel workers (default: number of CPU cores)
- `-b, --buffer-size <BYTES>`: Buffer size for reading files in bytes (default: 8192)

### Example

```bash
# Search for Python files containing "def main"
file-finder -n "\.py$" -c "def main" /path/to/project -C 2
```

## Performance

Benchmark results demonstrate efficient handling across different directory structures and file sizes:

### Directory Traversal Performance

| Structure | Depth | Files per Level | Search Time |
|-----------|-------|----------------|-------------|
| Shallow   | 2     | 5              | ~1.2ms      |
| Medium    | 4     | 3              | ~7.3ms      |
| Deep      | 6     | 2              | ~56ms       |

### Optimization Techniques

1. **Parallel Processing**: Uses Rayon for parallel file traversal and content searching
2. **Memory Mapping**: Employs mmap for files larger than 10MB
3. **Circular Buffer**: Optimized context line management with O(1) operations
4. **Efficient String Handling**: Pre-allocated buffers and minimal allocations
5. **Smart File Reading**: Buffered reading for small files, memory mapping for large ones