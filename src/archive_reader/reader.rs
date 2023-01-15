use super::iter;
use crate::error::{analyze_result, path_does_not_exist, Error, Result};
use log::{error, info};
use std::ffi::CString;
use std::path::Path;

use crate::libarchive;

pub struct ArchiveReader {
    handle: *mut libarchive::archive,
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
        info!("ArchiveReader::list_file_names_with_encoding()");
        iter::EntryIter::new(self.handle, decoding)
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
