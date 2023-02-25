use super::blocks::BlockReaderBorrowed;
use crate::error::{Error, Result};
use crate::lending_iter::LendingIterator;
use crate::libarchive;
use crate::locale::UTF8LocaleGuard;
use log::{error, info};
use std::borrow::Cow;
use std::ffi::CStr;
use std::io::Write;

pub struct Entry {
    archive: *mut libarchive::archive,
    entry: *mut libarchive::archive_entry,
}

unsafe impl Send for Entry {}

impl Entry {
    pub(crate) fn new(
        archive: *mut libarchive::archive,
        entry: *mut libarchive::archive_entry,
    ) -> Self {
        Self { archive, entry }
    }

    pub fn file_name<F>(&self, decode: F) -> Result<Cow<str>>
    where
        F: FnOnce(&[u8]) -> Option<Cow<str>>,
    {
        info!(r#"Entry::file_name(decode: _)"#);
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
        let entry_name_in_bytes = unsafe { CStr::from_ptr(entry_name).to_bytes() };
        match decode(entry_name_in_bytes) {
            Some(entry_name) => Ok(entry_name),
            None => {
                error!("failed to decode entry name");
                Err(Error::Encoding)
            }
        }
    }

    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(self) -> impl Iterator<Item = Result<Box<[u8]>>> + Send {
        info!(r#"Entry::read_file_by_block()"#);
        BlockReaderBorrowed::new(self.archive)
    }

    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block(
        self,
    ) -> impl for<'a> crate::LendingIterator<Item<'a> = Result<&'a [u8]>> + Send {
        info!(r#"Entry::read_file_by_block()"#);
        BlockReaderBorrowed::new(self.archive)
    }

    pub fn read_file<W: Write>(self, mut output: W) -> Result<usize> {
        info!(r#"Entry::read_file(output: _)"#);
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