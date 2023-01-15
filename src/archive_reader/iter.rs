use crate::error::{analyze_result, Error, Result};
use crate::{libarchive, ArchiveReader};
use log::{debug, error};
use std::ffi::CStr;
use std::slice;

pub(crate) struct EntryIter<F> {
    /// _reader_guard prevents the ArchiveReader from drop until the Iterator itself is dropped.
    _reader_guard: ArchiveReader,
    iterator: EntryIterBorrowed<F>,
}

impl<F> EntryIter<F>
where
    F: Fn(&[u8]) -> Option<String>,
{
    pub fn new(reader: ArchiveReader, decoding: F) -> Self {
        let iterator = EntryIterBorrowed::new(reader.handle, decoding);
        Self {
            _reader_guard: reader,
            iterator,
        }
    }
}

impl<F> Iterator for EntryIter<F>
where
    F: Fn(&[u8]) -> Option<String>,
{
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

pub(crate) struct EntryIterBorrowed<F> {
    handle: *mut libarchive::archive,
    decoding: F,
}

impl<F> EntryIterBorrowed<F>
where
    F: Fn(&[u8]) -> Option<String>,
{
    pub fn new(handle: *mut libarchive::archive, decoding: F) -> Self {
        Self { handle, decoding }
    }
}

impl<F> Iterator for EntryIterBorrowed<F>
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

pub struct BlockReader(ArchiveReader);

impl BlockReader {
    pub fn new(archive_reader: ArchiveReader) -> Self {
        BlockReader(archive_reader)
    }
}

impl Iterator for BlockReader {
    type Item = Result<Box<[u8]>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = std::ptr::null();
        let mut offset = 0;
        let mut size = 0;

        unsafe {
            match libarchive::archive_read_data_block(self.0.handle, &mut buf, &mut size, &mut offset) {
                1 /*libarchive::ARCHIVE_EOF*/ => return None,
                result => {
                    if let Err(error) = analyze_result(result, self.0.handle) {
                        return Some(Err(error))
                    }
                }
            };
            let content = slice::from_raw_parts(buf as *const u8, size);
            Some(Ok(content.into()))
        }
    }
}
