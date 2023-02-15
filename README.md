# archive-reader

`ArchiveReader` is a library that wraps partial read functions from libarchive.
It provides rustic interface over listing file names and reading given files within archives.

```toml
[dependencies]
archive-reader = "0.2"
```

# Example
```rust
use archive_reader::Archive;
use archive_reader::error::Result;

fn main() -> Result<()> {
    let mut archive = Archive::open("some_archive.zip");
    let file_names = archive
                        .block_size(1024 * 1024)
                        .list_file_names()?
                        .collect::<Result<Vec<_>>>()?;
    let mut content = vec![];
    let _ = archive.read_file(&file_names[0], &mut content)?;
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
* libc
* libarchive >= 3.2.0
  * Check it with command `pkg-config --libs --cflags libarchive 'libarchive >= 3.2.0'`

## Compile
```shell
cd SOME_DIR
git clone git@github.com:YaxinCheng/archive-reader.git
cd archive-reader
cargo build --release
```
# 
