use super::reader::ArchiveReader;
use crate::error::Result;
use crate::DecodingFn;
use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_BLOCK_SIZE: usize = 1024 * 1024;

pub struct Archive {
    block_size: usize,
    file_path: PathBuf,
    decoding_fn: Option<DecodingFn>,
}

impl Archive {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        Archive {
            block_size: DEFAULT_BLOCK_SIZE,
            file_path: path.as_ref().into(),
            decoding_fn: None,
        }
    }

    pub fn block_size(&mut self, block_size: usize) -> &mut Self {
        self.block_size = block_size;
        self
    }

    pub fn reset_block_size(&mut self) -> &mut Self {
        self.block_size = DEFAULT_BLOCK_SIZE;
        self
    }

    pub fn decoding_fn(&mut self, function: DecodingFn) -> &mut Self {
        self.decoding_fn = Some(function);
        self
    }

    pub fn reset_decoding_fn(&mut self) -> &mut Self {
        self.decoding_fn = None;
        self
    }
}

// Consumers
impl Archive {
    pub fn list_file_names(&self) -> Result<impl Iterator<Item = Result<String>>> {
        Ok(self
            .create_reader()?
            .list_file_names(self.get_decoding_fn()))
    }

    pub fn read_file<W: Write>(&self, file_name: &str, output: W) -> Result<usize> {
        self.create_reader()?
            .read_file(file_name, output, self.get_decoding_fn())
    }

    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(
        &self,
        file_name: &str,
    ) -> Result<impl Iterator<Item = Result<Box<[u8]>>> + Send> {
        self.create_reader()?
            .read_file_by_block(file_name, self.get_decoding_fn())
    }

    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block(
        &self,
        file_name: &str,
    ) -> Result<impl for<'a> crate::LendingIterator<Item<'a> = Result<&'a [u8]>> + Send> {
        self.create_reader()?
            .read_file_by_block(file_name, self.get_decoding_fn())
    }
}

// util functions
impl Archive {
    fn create_reader(&self) -> Result<ArchiveReader> {
        ArchiveReader::open(&self.file_path, self.block_size)
    }

    fn get_decoding_fn(&self) -> DecodingFn {
        match self.decoding_fn {
            Some(decoding_fn) => decoding_fn,
            None => Self::decode_utf8,
        }
    }

    fn decode_utf8(bytes: &[u8]) -> Option<Cow<'_, str>> {
        Some(String::from_utf8_lossy(bytes))
    }
}
