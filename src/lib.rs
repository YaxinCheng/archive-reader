//! `ArchiveReader` is a library that wraps partial read functions from libarchive.
//! It provides rustic interface over listing file names and reading given files within archives.
//!
//! # Example
//! ```rust,no_run
//! use archive_reader::Archive;
//! use archive_reader::error::Result;
//!
//! fn main() -> Result<()> {
//!     let mut archive = Archive::open("some_archive.zip");
//!     let file_names = archive
//!                         .block_size(1024 * 1024)
//!                         .list_file_names()?
//!                         .collect::<Result<Vec<_>>>()?;
//!     let mut content = vec![];
//!     let _ = archive.read_file(&file_names[0], &mut content)?;
//!     println!("content={content:?}");
//!     Ok(())
//! }
//! ```
//! # Features
//! * `lending_iter` - Enables `LendingIterator` implementation, which avoids heap allocations for `read_file_by_block` function.
//!

mod archive_reader;
pub mod error;
mod lending_iter;
mod libarchive;

pub use crate::archive_reader::*;
pub use error::*;
#[cfg(feature = "lending_iter")]
pub use lending_iter::LendingIterator;
#[cfg(not(feature = "lending_iter"))]
use lending_iter::LendingIterator;

type DecodingFn = fn(&[u8]) -> Option<std::borrow::Cow<'_, str>>;
