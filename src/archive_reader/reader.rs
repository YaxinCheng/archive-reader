use super::iter;
use crate::error::{analyze_result, path_does_not_exist, Error, Result};
use log::{error, info};
use std::ffi::CString;
use std::io::Write;
use std::path::Path;

use crate::libarchive;

pub type Bytes = Box<[u8]>;

pub struct ArchiveReader {
    pub(crate) handle: *mut libarchive::archive,
}

unsafe impl Send for ArchiveReader {}

const BLOCK_SIZE: usize = 16 * 1024;

impl ArchiveReader {
    pub fn open<P: AsRef<Path>>(archive_path: P) -> Result<Self> {
        let archive_path = archive_path.as_ref();
        info!(
            r#"ArchiveReader::open(archive_path: "{}")"#,
            archive_path.display()
        );
        if !archive_path.exists() {
            error!(r#"path "{}" does not exist"#, archive_path.display());
            return Err(path_does_not_exist(
                archive_path.to_string_lossy().to_string(),
            ));
        }
        let archive_path =
            CString::new(archive_path.to_str().ok_or(Error::PathNotUtf8)?).expect("Not null");
        unsafe {
            let handle = libarchive::archive_read_new();
            analyze_result(libarchive::archive_read_support_filter_all(handle), handle)?;
            analyze_result(libarchive::archive_read_support_format_raw(handle), handle)?;
            analyze_result(libarchive::archive_read_support_format_all(handle), handle)?;
            analyze_result(
                libarchive::archive_read_open_filename(handle, archive_path.as_ptr(), BLOCK_SIZE),
                handle,
            )?;
            Ok(ArchiveReader { handle })
        }
    }

    pub fn list_file_names(self) -> impl Iterator<Item = Result<String>> {
        info!("ArchiveReader::list_file_names()");
        self.list_file_names_with_encoding(|bytes| Some(String::from_utf8_lossy(bytes).to_string()))
    }

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

    pub fn read_file_to_bytes(self, file_name: &str) -> Result<Vec<u8>> {
        info!(r#"ArchiveReader::read_file_to_bytes("file_name: {file_name}")"#);
        let mut combined = Vec::new();
        self.read_file(file_name, &mut combined)?;
        Ok(combined)
    }

    pub fn read_file(self, file_name: &str, mut output: impl Write) -> Result<usize> {
        info!(r#"ArchiveReader::read_file("file_name: {file_name}", output: _)"#);
        let mut total_read = 0;
        for bytes in self.read_file_by_block(file_name)? {
            let bytes = bytes?;
            total_read += bytes.len();
            output.write_all(bytes.as_ref())?;
        }
        Ok(total_read)
    }

    pub fn read_file_to_bytes_with_encoding<F>(
        self,
        file_name: &str,
        decoding: F,
    ) -> Result<Vec<u8>>
    where
        F: Fn(&[u8]) -> Option<String>,
    {
        info!(
            r#"ArchiveReader::read_file_to_bytes_with_encoding("file_name: {file_name}", decoding: _)"#
        );
        let mut combined = Vec::new();
        self.read_file_with_encoding(file_name, &mut combined, decoding)?;
        Ok(combined)
    }

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
        for bytes in self.read_file_by_block_with_encoding(file_name, decoding)? {
            let bytes = bytes?;
            total_read += bytes.len();
            output.write_all(bytes.as_ref())?;
        }
        Ok(total_read)
    }

    pub fn read_file_by_block(
        self,
        file_name: &str,
    ) -> Result<impl Iterator<Item = Result<Bytes>>> {
        info!(r#"ArchiveReader::read_file_by_block("file_name: {file_name}")"#);
        self.read_file_by_block_with_encoding(file_name, |entry_name| {
            Some(String::from_utf8_lossy(entry_name).to_string())
        })
    }

    pub fn read_file_by_block_with_encoding<F>(
        self,
        file_name: &str,
        decoding: F,
    ) -> Result<impl Iterator<Item = Result<Bytes>>>
    where
        F: Fn(&[u8]) -> Option<String>,
    {
        info!(
            r#"ArchiveReader::read_file_by_blockwith_encoding(file_name: "{file_name}", decoding: _)"#
        );
        for entry_name in iter::EntryIterBorrowed::new(self.handle, decoding) {
            if entry_name? == file_name {
                break;
            }
        }
        Ok(iter::BlockReader::new(self))
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
            error!("Failed to clean up ArchiveReader: {error}")
        }
    }
}
