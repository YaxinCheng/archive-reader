//! `ArchiveReader` is a library that wraps partial read functions from libarchive.
//! It provides rustic interface over listing file names and reading given files within archives.
//!
//! # Example
//! ```rust,no_run
//! use archive_reader::ArchiveReader;
//! use archive_reader::error::Result;
//!
//! fn main() -> Result<()> {
//!     let file_names = ArchiveReader::open("some_archive.zip")?
//!                         .list_file_names()
//!                         .collect::<Result<Vec<_>>>()?;
//!     let mut content = vec![];
//!     let _ = ArchiveReader::open("some_archive.zip")?
//!                         .read_file(&file_names[0], &mut content)?;
//!     println!("content={content:?}");
//!     Ok(())
//! }
//! ```
//! # Features
//! * `lending_iter` - Enables `LendingIterator` implementation, which avoids heap allocations for `read_file_by_block` functions.
//!

mod archive_reader;
pub mod error;
#[cfg(feature = "lending_iter")]
mod lending_iter;
mod libarchive;

pub use crate::archive_reader::*;
pub use error::*;
#[cfg(feature = "lending_iter")]
pub use lending_iter::LendingIterator;
