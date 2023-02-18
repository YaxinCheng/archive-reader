use super::reader::ArchiveReader;
use crate::error::{analyze_result, Error, Result};
use crate::libarchive;
use crate::locale::{UTF8LocaleGuard, WindowsUTF8LocaleGuard};
use crate::{Decoder, LendingIterator};
use log::{debug, error};
use std::borrow::Cow;
use std::ffi::CStr;
use std::slice;

pub(crate) struct EntryIter {
    /// _reader_guard prevents the ArchiveReader from drop until the Iterator itself is dropped.
    _reader_guard: ArchiveReader,
    iterator: EntryIterBorrowed,
}

impl EntryIter {
    pub fn new(reader: ArchiveReader, decoding: Decoder) -> Self {
        let iterator = EntryIterBorrowed::new(reader.handle, decoding);
        Self {
            _reader_guard: reader,
            iterator,
        }
    }
}

impl Iterator for EntryIter {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iterator.next()?.map(String::from))
    }
}

pub(crate) struct EntryIterBorrowed {
    handle: *mut libarchive::archive,
    decoding: Decoder,
}

impl EntryIterBorrowed {
    pub fn new(handle: *mut libarchive::archive, decoding: Decoder) -> Self {
        Self { handle, decoding }
    }
}

impl LendingIterator for EntryIterBorrowed {
    type Item<'a> = Result<Cow<'a, str>>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        debug_assert!(!self.handle.is_null(), "EntryIterBorrowed::handle is null");
        let _locale_guard = UTF8LocaleGuard::new();
        let mut entry = std::ptr::null_mut();
        match unsafe { libarchive::archive_read_next_header(self.handle, &mut entry) } {
            libarchive::ARCHIVE_EOF => {
                debug!("archive_read_next_header: reaches EOF");
                return None;
            }
            result => {
                if let Err(error) = analyze_result(result, self.handle) {
                    error!("archive_read_next_header error: {error:?}");
                    return Some(Err(error));
                }
                debug!("archive_read_next_header: success");
            }
        };
        let _locale_guard = WindowsUTF8LocaleGuard::new();
        let entry_name = unsafe { libarchive::archive_entry_pathname(entry) };
        if entry_name.is_null() {
            error!("archive_entry_pathname returns null");
            return Some(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "archive entry contains invalid name".to_string(),
            )
            .into()));
        }
        let entry_name_in_bytes = unsafe { CStr::from_ptr(entry_name).to_bytes() };
        match (self.decoding)(entry_name_in_bytes) {
            Some(entry_name) => Some(Ok(entry_name)),
            None => {
                error!("failed to decode entry name");
                Some(Err(Error::Encoding))
            }
        }
    }
}

/// `BlockReader` is an iterator that reads an archive entry block by block.
pub(crate) struct BlockReader {
    reader: ArchiveReader,
    /// ended is set to true when the iterator has reached its end.
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
        match unsafe {
            libarchive::archive_read_data_block(
                self.reader.handle,
                &mut buf,
                &mut size,
                &mut offset,
            )
        } {
            libarchive::ARCHIVE_EOF => {
                debug!("archive_read_data_block: reaches eof");
                self.ended = true;
                Ok(&[])
            }
            result => match analyze_result(result, self.reader.handle) {
                Ok(()) => {
                    let content = unsafe { slice::from_raw_parts(buf as *const u8, size) };
                    Ok(content)
                }
                Err(error) => {
                    error!("archive_read_data_block error: {error:?}");
                    self.ended = true;
                    Err(error)
                }
            },
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
