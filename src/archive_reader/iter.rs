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
        if self.handle.is_null() {
            return None;
        }
        let mut entry = std::ptr::null_mut();
        unsafe {
            match libarchive::archive_read_next_header(self.handle, &mut entry) {
                libarchive::ARCHIVE_EOF => {
                    self.handle = std::ptr::null_mut();
                    debug!("archive_read_next_header: reaches EOF");
                    return None;
                }
                result => {
                    if let Err(error) = analyze_result(result, self.handle) {
                        error!("archive_read_next_header error: {error}");
                        return Some(Err(error));
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

pub struct BlockReader {
    reader: ArchiveReader,
    ended: bool,
}

impl BlockReader {
    pub fn new(archive_reader: ArchiveReader) -> Self {
        BlockReader {
            reader: archive_reader,
            ended: false,
        }
    }

    pub fn read_block(&mut self) -> Result<&[u8]> {
        if self.ended {
            return Ok(&[]);
        }
        let mut buf = std::ptr::null();
        let mut offset = 0;
        let mut size = 0;

        unsafe {
            match libarchive::archive_read_data_block(
                self.reader.handle,
                &mut buf,
                &mut size,
                &mut offset,
            ) {
                libarchive::ARCHIVE_EOF => {
                    debug!("archive_read_data_block: reaches eof");
                    self.ended = true;
                    Ok(&[])
                }
                result => match analyze_result(result, self.reader.handle) {
                    Ok(()) => {
                        let content = slice::from_raw_parts(buf as *const u8, size);
                        Ok(content)
                    }
                    Err(error) => {
                        error!("archive_read_data_block error: {error}");
                        Err(error)
                    }
                },
            }
        }
    }
}

#[cfg(not(feature = "lending_iter"))]
impl Iterator for BlockReader {
    type Item = Result<Box<[u8]>>;

    fn next(&mut self) -> Option<Result<Box<[u8]>>> {
        match self.read_block() {
            Ok(&[]) => None,
            block => Some(block.map(Box::from)),
        }
    }
}

#[cfg(feature = "lending_iter")]
impl crate::LendingIterator for BlockReader {
    type Item<'me> = Result<&'me [u8]>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        match self.read_block() {
            Ok(&[]) => None,
            block => Some(block),
        }
    }
}
