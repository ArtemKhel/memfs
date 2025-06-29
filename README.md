# MemFS - In-Memory File System

An async in-memory file system for Rust. Supports basic file operations without directories.

## Features

- Async operations with Tokio
- Thread-safe concurrent access
- Fine-grained locking
- Simple API: `touch`, `write`, `read`

## Usage

```rust
use memfs::FileSystem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs = FileSystem::new();

    // Create and write to file
    fs.touch("/log.txt").await?;
    fs.write("/log.txt", 0, b"hello").await?;
    fs.write("/log.txt", 5, b" world").await?;

    // Read data
    let content = fs.read("/log.txt", 0, 11).await?;
    assert_eq!(content, b"hello world");

    Ok(())
}
```

## API

- `touch(path)` - Create file if it doesn't exist
- `write(path, offset, data)` - Write data at offset, extends file if needed
- `read(path, offset, len)` - Read data, returns available bytes

## Testing

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
