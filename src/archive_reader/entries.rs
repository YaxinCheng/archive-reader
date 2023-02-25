use super::entry::Entry;
use crate::error::{analyze_result, path_does_not_exist, Error, Result};
use crate::libarchive;
use log::{debug, error, info};
use std::ffi::CString;
use std::path::Path;

#[cfg(feature = "lending_iter")]
use crate::LendingIterator;

#[cfg(not(feature = "lending_iter"))]
pub(crate) struct Entries {
    pub(crate) archive: *mut libarchive::archive,
}

#[cfg(feature = "lending_iter")]
pub(crate) struct Entries {
    pub(crate) archive: *mut libarchive::archive,
    pub(crate) entry: Option<Entry>,
}

unsafe impl Send for Entries {}

#[cfg(not(feature = "lending_iter"))]
impl Iterator for Entries {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = unsafe { self.read_entry() }?;
        match entry {
            Ok(entry) => Some(Ok(Entry::new(self.archive, entry))),
            Err(error) => Some(Err(error)),
        }
    }
}

#[cfg(feature = "lending_iter")]
impl LendingIterator for Entries {
    type Item<'me> = Result<&'me mut Entry>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let entry = unsafe { self.read_entry() }?;
        let entry = match entry {
            Err(error) => return Some(Err(error)),
            Ok(entry) => Entry::new(self.archive, entry),
        };
        self.entry.replace(entry);
        self.entry.as_mut().map(Ok)
    }
}

impl Entries {
    unsafe fn read_entry(&self) -> Option<Result<*mut libarchive::archive_entry>> {
        let mut entry = std::ptr::null_mut();
        match libarchive::archive_read_next_header(self.archive, &mut entry) {
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
        Some(Ok(entry))
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
            #[cfg(feature = "lending_iter")]
            entry: None,
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

    pub(crate) fn file_names(self) -> EntryNames {
        info!(r#"Entries::file_names(decoder: _)"#);
        EntryNames(self)
    }

    pub(crate) fn find_entry_by_name<P>(&mut self, predicate: P) -> Result<()>
    where
        P: Fn(&[u8]) -> bool,
    {
        info!(r#"Entries::find_entry_by_name(predicate: _)"#);
        while let Some(item) = self.next() {
            match item {
                Ok(entry) if predicate(entry.file_name()?) => return Ok(()),
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

pub(crate) struct EntryNames(Entries);

impl Iterator for EntryNames {
    type Item = Result<bytes::Bytes>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = match self.0.next()? {
            Ok(entry) => entry.file_name().map(bytes::Bytes::copy_from_slice),
            Err(error) => Err(error),
        };
        Some(name)
    }
}
