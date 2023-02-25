use super::entries::Entries;
use crate::error::{analyze_result, Result};
use crate::libarchive;
use crate::LendingIterator;
use log::{debug, error};
use std::slice;

/// `BlockReader` is an iterator that reads an archive entry block by block.
pub(crate) struct BlockReader {
    _entries: Entries, // Kept in the structure to prevent it from being dropped.
    block_reader: BlockReaderBorrowed,
}

impl BlockReader {
    pub fn new(entries: Entries) -> Self {
        let block_reader = BlockReaderBorrowed::from(&entries);
        BlockReader {
            _entries: entries,
            block_reader,
        }
    }
}

#[cfg(not(feature = "lending_iter"))]
impl Iterator for BlockReader {
    type Item = Result<Box<[u8]>>;

    fn next(&mut self) -> Option<Result<Box<[u8]>>> {
        Iterator::next(&mut self.block_reader)
    }
}

#[cfg(feature = "lending_iter")]
impl LendingIterator for BlockReader {
    type Item<'me> = Result<&'me [u8]>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        crate::LendingIterator::next(&mut self.block_reader)
    }
}

pub(crate) struct BlockReaderBorrowed {
    archive: *mut libarchive::archive,
    /// ended is set to true when the iterator has reached its end.
    ended: bool,
}

unsafe impl Send for BlockReaderBorrowed {}

impl From<&Entries> for BlockReaderBorrowed {
    fn from(entries: &Entries) -> Self {
        BlockReaderBorrowed::new(entries.archive)
    }
}

impl BlockReaderBorrowed {
    pub(crate) fn new(archive: *mut libarchive::archive) -> Self {
        Self {
            archive,
            ended: false,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            archive: std::ptr::null_mut(),
            ended: true,
        }
    }

    pub(crate) fn read_block(&mut self) -> Result<&[u8]> {
        if self.ended {
            return Ok(&[]);
        }
        let mut buf = std::ptr::null();
        let mut offset = 0;
        let mut size = 0;
        match unsafe {
            libarchive::archive_read_data_block(self.archive, &mut buf, &mut size, &mut offset)
        } {
            libarchive::ARCHIVE_EOF => {
                debug!("archive_read_data_block: reaches eof");
                self.ended = true;
                Ok(&[])
            }
            result => match analyze_result(result, self.archive) {
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

impl Iterator for BlockReaderBorrowed {
    type Item = Result<Box<[u8]>>;

    fn next(&mut self) -> Option<Result<Box<[u8]>>> {
        match self.read_block() {
            Ok(&[]) => None,
            block => Some(block.map(Box::from)),
        }
    }
}

impl LendingIterator for BlockReaderBorrowed {
    type Item<'me> = Result<&'me [u8]>;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        match self.read_block() {
            Ok(&[]) => None,
            block => Some(block),
        }
    }
}
