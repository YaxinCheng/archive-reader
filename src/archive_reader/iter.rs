use crate::error::{analyze_result, Error, Result};
use crate::libarchive;
use log::{debug, error};
use std::ffi::CStr;

pub struct EntryIter<F> {
    handle: *mut libarchive::archive,
    decoding: F,
}

impl<F> EntryIter<F>
where
    F: Fn(&[u8]) -> Option<String>,
{
    pub fn new(handle: *mut libarchive::archive, decoding: F) -> Self {
        Self { handle, decoding }
    }
}

impl<F> Iterator for EntryIter<F>
where
    F: Fn(&[u8]) -> Option<String>,
{
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut entry = std::ptr::null_mut();
        unsafe {
            match libarchive::archive_read_next_header(self.handle, &mut entry) {
                1 /*libarchive::ARCHIVE_EOF*/ => {
                    debug!("archive_read_next_header: reaches EOF");
                    return None
                },
                result => {
                    if let Err(error) = analyze_result(result, self.handle) {
                        error!("archive_read_next_header error: {error}");
                        return Some(Err(error))
                    }
                    debug!("archive_read_next_header: success");
                }
            };
            let entry_name = libarchive::archive_entry_pathname(entry);
            if entry_name.is_null() {
                error!("archive_entry_pathname returns null");
                return Some(Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "archive entry contains invalid name".to_string(),
                )
                .into()));
            }
            match (self.decoding)(CStr::from_ptr(entry_name).to_bytes()) {
                Some(entry_name) => Some(Ok(entry_name)),
                None => {
                    error!("failed to decode entry name");
                    Some(Err(Error::Encoding))
                }
            }
        }
    }
}
