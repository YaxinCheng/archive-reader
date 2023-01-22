use super::iter;
use crate::error::{analyze_result, path_does_not_exist, Error, Result};
use log::{error, info};
use std::ffi::CString;
use std::io::Write;
use std::path::Path;

use crate::libarchive;
#[cfg(feature = "lending_iter")]
use crate::LendingIterator;

/// `ArchiveReader` is a type that handles the archive reading task.
/// It wraps partial functionalities of libarchive to read archives.
///
/// # Note:
/// As libarchive does not support random access,
/// every function in ArchiveReader consumes itself.
pub struct ArchiveReader {
    pub(crate) handle: *mut libarchive::archive,
}

// pointer to an libarchive struct is safe to move.
unsafe impl Send for ArchiveReader {}

/// BLOCK_SIZE is the size for each block to be read in through ArchiveReader.
const BLOCK_SIZE: usize = 16 * 1024;

impl ArchiveReader {
    /// `open` is the constructor for ArchiveReader.
    /// It takes in the path to the archive.
    pub fn open<P: AsRef<Path>>(archive_path: P) -> Result<Self> {
        let archive_path = archive_path.as_ref();
        info!(
            r#"ArchiveReader::open(archive_path: "{}")"#,
            archive_path.display()
        );
        Self::path_exists(archive_path)?;
        let handle = Self::create_handle(archive_path)?;
        Ok(ArchiveReader { handle })
    }

    fn path_exists(archive_path: &Path) -> Result<()> {
        if !archive_path.exists() {
            error!(r#"path "{}" does not exist"#, archive_path.display());
            return Err(path_does_not_exist(
                archive_path.to_string_lossy().to_string(),
            ));
        }
        Ok(())
    }

    fn create_handle(archive_path: &Path) -> Result<*mut libarchive::archive> {
        let archive_path = CString::new(archive_path.to_str().ok_or(Error::PathNotUtf8)?)
            .expect("An existing path cannot be null");
        unsafe {
            let handle = libarchive::archive_read_new();
            analyze_result(libarchive::archive_read_support_filter_all(handle), handle)?;
            analyze_result(libarchive::archive_read_support_format_raw(handle), handle)?;
            analyze_result(libarchive::archive_read_support_format_all(handle), handle)?;
            analyze_result(
                libarchive::archive_read_open_filename(handle, archive_path.as_ptr(), BLOCK_SIZE),
                handle,
            )?;
            Ok(handle)
        }
    }

    /// `list_file_names` extracts file names from the target archive using UTF8 encoding.
    pub fn list_file_names(self) -> impl Iterator<Item = Result<String>> {
        info!("ArchiveReader::list_file_names()");
        self.list_file_names_with_encoding(|bytes| Some(String::from_utf8_lossy(bytes).to_string()))
    }

    /// `list_file_names_with_encoding` extracts file names from the target archive
    /// using custom decoding function.
    pub fn list_file_names_with_encoding<F>(
        self,
        decoding: F,
    ) -> impl Iterator<Item = Result<String>>
    where
        F: Fn(&[u8]) -> Option<String>,
    {
        info!("ArchiveReader::list_file_names_with_encoding(decoding: _)");
        iter::EntryIter::new(self, decoding)
    }

    /// `read_file` locates a file based on its file name in UTF8 and reads its content
    /// into a provided writable output.
    /// Eventually, it returns the size for total bytes read.
    pub fn read_file(self, file_name: &str, mut output: impl Write) -> Result<usize> {
        info!(r#"ArchiveReader::read_file("file_name: {file_name}", output: _)"#);
        let mut total_read = 0;
        let mut blocks = self.read_file_by_block(file_name)?;
        while let Some(bytes) = blocks.next() {
            let bytes = bytes?;
            total_read += bytes.len();
            output.write_all(bytes.as_ref())?;
        }
        Ok(total_read)
    }

    /// `read_file_with_encoding` locates a file based on its file name
    /// in provided encoding and reads its content
    /// into a provided writable output.
    /// Eventually, it returns the size for total bytes read.
    pub fn read_file_with_encoding<W, F>(
        self,
        file_name: &str,
        mut output: W,
        decoding: F,
    ) -> Result<usize>
    where
        W: Write,
        F: Fn(&[u8]) -> Option<String>,
    {
        info!(
            r#"ArchiveReader::read_file_with_encoding("file_name: {file_name}", output: _, decoding: _)"#
        );
        let mut total_read = 0;
        let mut blocks = self.read_file_by_block_with_encoding(file_name, decoding)?;
        while let Some(bytes) = blocks.next() {
            let bytes = bytes?;
            total_read += bytes.len();
            output.write_all(bytes.as_ref())?;
        }
        Ok(total_read)
    }

    /// `read_file_by_block` locates a file based on its UTF8 encoded file name,
    /// and reads its content as an iterator of blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(
        self,
        file_name: &str,
    ) -> Result<impl Iterator<Item = Result<Box<[u8]>>> + Send> {
        info!(r#"ArchiveReader::read_file_by_block("file_name: {file_name}")"#);
        self.read_file_by_block_with_encoding(file_name, |entry_name| {
            Some(String::from_utf8_lossy(entry_name).to_string())
        })
    }

    /// `read_file_by_block` locates a file based on its UTF8 encoded file name,
    /// and reads its content as a lending iterator of blocks.
    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block(
        self,
        file_name: &str,
    ) -> Result<impl for<'a> LendingIterator<Item<'a> = Result<&'a [u8]>> + Send> {
        info!(r#"ArchiveReader::read_file_by_block("file_name: {file_name}")"#);
        self.read_file_by_block_with_encoding(file_name, |entry_name| {
            Some(String::from_utf8_lossy(entry_name).to_string())
        })
    }

    /// `read_file_by_block_with_encoding` locates a file based on its file name
    /// with custom decoding function,
    /// and reads its content as a lending iterator of blocks.
    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block_with_encoding<F>(
        self,
        file_name: &str,
        decoding: F,
    ) -> Result<impl for<'a> LendingIterator<Item<'a> = Result<&'a [u8]>> + Send>
    where
        F: Fn(&[u8]) -> Option<String>,
    {
        self.read_file_by_block_with_encoding_raw(file_name, decoding)
    }

    /// `read_file_by_block_with_encoding` locates a file based on its file name
    /// with custom decoding function,
    /// and reads its content as an iterator of blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block_with_encoding<F>(
        self,
        file_name: &str,
        decoding: F,
    ) -> Result<impl Iterator<Item = Result<Box<[u8]>>> + Send>
    where
        F: Fn(&[u8]) -> Option<String>,
    {
        self.read_file_by_block_with_encoding_raw(file_name, decoding)
    }

    fn read_file_by_block_with_encoding_raw<F>(
        self,
        file_name: &str,
        decoding: F,
    ) -> Result<iter::BlockReader>
    where
        F: Fn(&[u8]) -> Option<String>,
    {
        info!(
            r#"ArchiveReader::read_file_by_block_with_encoding(file_name: "{file_name}", decoding: _)"#
        );
        let found = iter::EntryIterBorrowed::new(self.handle, decoding)
            .find(|entry_name| {
                entry_name
                    .as_ref()
                    .map(|name| name == file_name)
                    .unwrap_or_default()
            })
            .transpose()?;
        if found.is_some() {
            Ok(iter::BlockReader::new(self))
        } else {
            Err(Error::NotFound(file_name.to_string()))
        }
    }

    fn clean(&self) -> Result<()> {
        info!("ArchiveReader::clean()");
        unsafe {
            analyze_result(libarchive::archive_read_close(self.handle), self.handle)?;
            analyze_result(libarchive::archive_read_free(self.handle), self.handle)
        }
    }
}

impl Drop for ArchiveReader {
    fn drop(&mut self) {
        if let Err(error) = self.clean() {
            error!("Failed to clean up ArchiveReader: {error:?}")
        }
    }
}
