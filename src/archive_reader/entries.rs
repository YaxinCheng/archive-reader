use super::reader::ArchiveReader;
use crate::error::{analyze_result, Error, Result};
use crate::lending_iter::LendingIterator;
use crate::libarchive;
use crate::locale::{UTF8LocaleGuard, WindowsUTF8LocaleGuard};
use log::{debug, error};
use std::borrow::Cow;
use std::ffi::CStr;

pub struct Entries {
    reader: ArchiveReader,
    current_entry: Option<Entry>,
}

impl LendingIterator for Entries {
    type Item<'me> = Result<&'me Entry>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let mut entry = std::ptr::null_mut();
        match unsafe { libarchive::archive_read_next_header(self.reader.handle, &mut entry) } {
            libarchive::ARCHIVE_EOF => {
                debug!("archive_read_next_header: reaches EOF");
                return None;
            }
            result => {
                if let Err(error) = analyze_result(result, self.reader.handle) {
                    error!("archive_read_next_header error: {error:?}");
                    return Some(Err(error));
                }
                debug!("archive_read_next_header: success");
            }
        };
        self.current_entry.replace(Entry(entry));
        self.current_entry.as_ref().map(Ok)
    }
}

pub struct Entry(*mut libarchive::archive_entry);

impl Entry {
    pub fn file_name<F>(&self, decode: F) -> Result<Cow<str>>
    where
        F: FnOnce(&[u8]) -> Option<Cow<str>>,
    {
        let _utf8_locale_guard = UTF8LocaleGuard::new();
        let _windows_locale_guard = WindowsUTF8LocaleGuard::new();

        let entry_name = unsafe { libarchive::archive_entry_pathname(self.0) };
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
}
