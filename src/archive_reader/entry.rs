use super::blocks::BlockReaderBorrowed;
use super::entries::Entries;
use crate::error::{invalid_data, Result};
use crate::lending_iter::LendingIterator;
use crate::locale::UTF8LocaleGuard;
use crate::{libarchive, Error};
use log::{error, info};
use std::borrow::Cow;
use std::ffi::CStr;
use std::io::Write;

/// `Entry` represents a file / dir in an archive.
pub struct Entry<'a> {
    entries: &'a Entries,
    entry: *mut libarchive::archive_entry,
}

impl<'a> Entry<'a> {
    pub(crate) fn new(entries: &'a Entries, entry: *mut libarchive::archive_entry) -> Self {
        Self { entries, entry }
    }

    /// `file_name` returns the name of the entry decoded with the provided decoder.
    /// It may fail if the decoder cannot decode the name.
    pub fn file_name(&self) -> Result<Cow<str>> {
        info!(r#"Entry::file_name()"#);
        let _utf8_locale_guard = UTF8LocaleGuard::new();

        let entry_name = unsafe { libarchive::archive_entry_pathname(self.entry) };
        if entry_name.is_null() {
            error!("archive_entry_pathname returns null");
            return Err(invalid_data("archive entry contains invalid name"));
        }
        let entry_name_in_bytes = unsafe { CStr::from_ptr(entry_name).to_bytes() };
        match (self.entries.decoder)(entry_name_in_bytes) {
            Some(entry_name) => Ok(entry_name),
            None => {
                error!("failed to decode entry name");
                Err(Error::Encoding)
            }
        }
    }

    /// `read_file_by_block` returns an iterator of the entry content blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(self) -> impl Iterator<Item = Result<bytes::Bytes>> + Send + 'a {
        info!(r#"Entry::read_file_by_block()"#);
        BlockReaderBorrowed::from(self.entries)
    }

    /// `read_file_by_block` returns an iterator of the entry content blocks.
    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block(
        self,
    ) -> impl for<'b> LendingIterator<Item<'b> = Result<&'b [u8]>> + Send + 'a {
        info!(r#"Entry::read_file_by_block()"#);
        BlockReaderBorrowed::from(self.entries)
    }

    /// `read_file` reads the content of this entry to an output.
    pub fn read_file<W: Write>(self, mut output: W) -> Result<usize> {
        info!(r#"Entry::read_file(output: _)"#);
        let mut blocks = BlockReaderBorrowed::from(self.entries);
        let mut written = 0;
        while let Some(block) = LendingIterator::next(&mut blocks) {
            let block = block?;
            written += block.len();
            output.write_all(block)?;
        }
        Ok(written)
    }
}
