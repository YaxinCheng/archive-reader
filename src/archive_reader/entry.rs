use crate::archive_reader::iter;
use crate::error::{analyze_result, Error, Result};
use crate::libarchive;
use crate::locale::UTF8LocaleGuard;
use log::error;
use std::borrow::Cow;
use std::ffi::CStr;
use std::io::Write;

pub struct Entry(pub(crate) *mut libarchive::archive_entry);

unsafe impl Send for Entry {}

impl Entry {
    pub fn file_name<F>(&self, decode: F) -> Result<Cow<str>>
    where
        F: FnOnce(&[u8]) -> Option<Cow<str>>,
    {
        let _utf8_locale_guard = UTF8LocaleGuard::new();

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
