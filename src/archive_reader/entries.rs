use super::entry::Entry;
use crate::error::{analyze_result, path_does_not_exist, Error, Result};
use crate::lending_iter::LendingIterator;
use crate::{libarchive, Decoder};
use log::{debug, error, info};
use std::ffi::CString;
use std::path::Path;

pub struct Entries {
    pub(crate) archive: *mut libarchive::archive,
    current_entry: Option<Entry>,
}

unsafe impl Send for Entries {}

impl LendingIterator for Entries {
    type Item<'me> = Result<&'me Entry>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let mut entry = std::ptr::null_mut();
        match unsafe { libarchive::archive_read_next_header(self.archive, &mut entry) } {
            libarchive::ARCHIVE_EOF => {
                debug!("archive_read_next_header: reaches EOF");
                return None;
            }
            result => {
                if let Err(error) = analyze_result(result, self.archive) {
                    error!("archive_read_next_header error: {error:?}");
                    return Some(Err(error));
                }
                debug!("archive_read_next_header: success");
            }
        };
        self.current_entry.replace(Entry::new(self.archive, entry));
        self.current_entry.as_ref().map(Ok)
    }
}

impl Entries {
    /// `open` is the constructor for ArchiveReader.
    /// It takes in the path to the archive.
    pub(crate) fn open<P: AsRef<Path>>(archive_path: P, block_size: usize) -> Result<Self> {
        let archive_path = archive_path.as_ref();
        info!(
            r#"ArchiveReader::open(archive_path: "{}")"#,
            archive_path.display()
        );
        Self::path_exists(archive_path)?;
        let archive = Self::create_handle(archive_path, block_size)?;
        Ok(Entries {
            archive,
            current_entry: None,
        })
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

    fn create_handle(archive_path: &Path, block_size: usize) -> Result<*mut libarchive::archive> {
        let archive_path = CString::new(archive_path.to_str().ok_or(Error::PathNotUtf8)?)
            .expect("An existing path cannot be null");
        unsafe {
            let handle = libarchive::archive_read_new();
            analyze_result(libarchive::archive_read_support_filter_all(handle), handle)?;
            analyze_result(libarchive::archive_read_support_format_raw(handle), handle)?;
            analyze_result(libarchive::archive_read_support_format_all(handle), handle)?;
            analyze_result(
                libarchive::archive_read_open_filename(handle, archive_path.as_ptr(), block_size),
                handle,
            )?;
            Ok(handle)
        }
    }

    fn clean(&self) -> Result<()> {
        info!("Entries::clean()");
        unsafe {
            analyze_result(libarchive::archive_read_close(self.archive), self.archive)?;
            analyze_result(libarchive::archive_read_free(self.archive), self.archive)
        }
    }

    pub(crate) fn file_names(
        self,
        decoder: Decoder,
    ) -> impl Iterator<Item = Result<String>> + Send {
        info!(r#"Entries::file_names(decoder: _)"#);
        EntryNames {
            entries: self,
            decoder,
        }
    }

    pub(crate) fn find_entry_by_name(&mut self, decoder: Decoder, file_name: &str) -> Result<()> {
        info!(r#"Entries::find_entry_by_name(decoder: _, file_name: "{file_name}")"#);
        while let Some(item) = self.next() {
            match item {
                Ok(entry) if entry.file_name(decoder)? == file_name => return Ok(()),
                Err(error) => return Err(error),
                _ => (),
            }
        }
        Err(path_does_not_exist("find_entry_failed".to_string()))
    }
}

impl Drop for Entries {
    fn drop(&mut self) {
        if let Err(error) = self.clean() {
            error!("Failed to clean up Entries: {error:?}")
        }
    }
}

pub(crate) struct EntryNames {
    entries: Entries,
    decoder: Decoder,
}

impl Iterator for EntryNames {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = match self.entries.next()? {
            Ok(entry) => entry.file_name(self.decoder).map(String::from),
            Err(error) => Err(error),
        };
        Some(name)
    }
}
