use crate::error::{analyze_result, Result};
use crate::{libarchive, Entries};
use log::{debug, error};
use std::slice;

/// `BlockReader` is an iterator that reads an archive entry block by block.
pub(crate) struct BlockReader {
    entries: Entries,
    /// ended is set to true when the iterator has reached its end.
    ended: bool,
}

unsafe impl Send for BlockReader {}

impl BlockReader {
    pub fn new(entries: Entries) -> Self {
        BlockReader {
            entries,
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
                self.entries.archive,
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
            result => match analyze_result(result, self.entries.archive) {
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
