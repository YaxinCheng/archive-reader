# archive-reader

`ArchiveReader` is a library that wraps partial read functions from libarchive.
It provides rustic interface over listing file names and reading given files within archives.

```toml
[dependencies]
archive-reader = "0.1"
```

# Example
```rust
use archive_reader::ArchiveReader;
use archive_reader::error::Result;

fn main() -> Result<()> {
    let file_names = ArchiveReader::open("some_archive.zip")?
                        .list_file_names()
                        .collect::<Result<Vec<_>>>()?;
    let mut content = vec![];
    let _ = ArchiveReader::open("some_archive.zip")?
                        .read_file(&file_names[0], &mut content)?;
    println!("content={:?}", content);
    Ok(())
}
```

# Features
* `lending_iter` - Enables `LendingIterator` implementation, which avoids heap allocations for `read_file_by_block` functions.

# Getting Started
This section talks about compiling this project
## Prerequisites:
* Rust 1.66.0 (May be compatible with lower versions, but I used 1.66.0)
* Cargo
* Git

## Compile
```shell
cd SOME_DIR
git clone git@github.com:YaxinCheng/archive-reader.git
cd archive-reader
cargo build --release
```
# 
