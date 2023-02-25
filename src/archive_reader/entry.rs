use super::blocks::BlockReaderBorrowed;
use crate::error::Result;
use crate::lending_iter::LendingIterator;
use crate::libarchive;
use crate::locale::UTF8LocaleGuard;
use bytes::Bytes;
use log::{error, info};
use std::ffi::CStr;
use std::io::Write;

/// `Entry` represents a file / dir in an archive.
pub struct Entry {
    archive: *mut libarchive::archive,
    entry: *mut libarchive::archive_entry,
    already_read: bool,
}

unsafe impl Send for Entry {}

impl Entry {
    pub(crate) fn new(
        archive: *mut libarchive::archive,
        entry: *mut libarchive::archive_entry,
    ) -> Self {
        Self {
            archive,
            entry,
            already_read: false,
        }
    }

    /// `file_name` returns the name of the entry decoded with the provided decoder.
    /// It may fail if the decoder cannot decode the name.
    pub fn file_name(&self) -> Result<&[u8]> {
        info!(r#"Entry::file_name()"#);
        let _utf8_locale_guard = UTF8LocaleGuard::new();

        let entry_name = unsafe { libarchive::archive_entry_pathname(self.entry) };
        if entry_name.is_null() {
            error!("archive_entry_pathname returns null");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "archive entry contains invalid name".to_string(),
            )
            .into());
        }
        unsafe { Ok(CStr::from_ptr(entry_name).to_bytes()) }
    }

    /// `read_file_by_block` returns an iterator of the entry content blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(&mut self) -> impl Iterator<Item = Result<Bytes>> + Send {
        info!(r#"Entry::read_file_by_block()"#);
        if self.already_read {
            BlockReaderBorrowed::empty()
        } else {
            self.already_read = true;
            BlockReaderBorrowed::new(self.archive)
        }
    }

    /// `read_file_by_block` returns an iterator of the entry content blocks.
    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block(
        &mut self,
    ) -> impl for<'a> crate::LendingIterator<Item<'a> = Result<&'a [u8]>> + Send {
        info!(r#"Entry::read_file_by_block()"#);
        if self.already_read {
            BlockReaderBorrowed::empty()
        } else {
            self.already_read = true;
            BlockReaderBorrowed::new(self.archive)
        }
    }

    /// `read_file` reads the content of this entry to an output.
    pub fn read_file<W: Write>(&mut self, mut output: W) -> Result<usize> {
        info!(r#"Entry::read_file(output: _)"#);
        if self.already_read {
            return Ok(0);
        }
        let mut blocks = BlockReaderBorrowed::new(self.archive);
        let mut written = 0;
        while let Some(block) = LendingIterator::next(&mut blocks) {
            let block = block?;
            written += block.len();
            output.write_all(block)?;
        }
        Ok(written)
    }
}
