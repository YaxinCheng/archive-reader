use super::reader::ArchiveReader;
use crate::error::Result;
use crate::Decoder;
use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_BLOCK_SIZE: usize = 1024 * 1024;

/// `Archive` represents an archive file which can be processed.
pub struct Archive {
    /// `block_size` is a size that will be used to break down content into blocks.
    /// The blocks read from the archive are not exactly the size of the `block_size`,
    /// due to compression and other factors.
    /// Increasing the size of this variable can make the reader reads more content
    /// into each block.
    block_size: usize,
    /// `file_path` is the path to the target archive.
    file_path: PathBuf,
    /// `decoder` is a function that decodes bytes into a proper string.
    /// By default, it decodes using UTF8.
    decoder: Option<Decoder>,
}

impl Archive {
    /// `open` creates a default `Archive` configuration from the given path.
    ///
    /// # Note:
    /// It handles the path lazily. So no error will occur until the path is used
    /// and proved to be problematic.
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        Archive {
            block_size: DEFAULT_BLOCK_SIZE,
            file_path: path.as_ref().into(),
            decoder: None,
        }
    }

    /// `block_size` sets the size limit for every block reading from the archive.
    /// The block size is represented in bytes.
    ///
    /// # Note:
    /// Content from archives are read block by block.
    /// Setting the block size will increase/decrease the time and content of
    /// reading each block.  
    pub fn block_size(&mut self, block_size: usize) -> &mut Self {
        self.block_size = block_size;
        self
    }

    /// `reset_block_size` resets the block size back to the default value (1024 * 1024).
    pub fn reset_block_size(&mut self) -> &mut Self {
        self.block_size(DEFAULT_BLOCK_SIZE)
    }

    /// `decoding_fn` sets a function as the decoder.
    ///
    /// # Note:
    /// A decoder is a function that converts a series of bytes into a proper string.
    /// In the case where the conversion failed, it should return `None`.
    pub fn decoder(&mut self, function: Decoder) -> &mut Self {
        self.decoder = Some(function);
        self
    }

    /// `reset_decoder` resets the decoder back to the default decoder.
    /// The default decoder converts the bytes into an UTF-8 encoded string.
    /// Any inconvertible characters will be replaced with a
    /// U+FFFD REPLACEMENT CHARACTER, which looks like this: �.
    pub fn reset_decoder(&mut self) -> &mut Self {
        self.decoder = None;
        self
    }
}

// Consumers
impl Archive {
    /// `list_file_names` return an iterator of file names extracted from the archive.
    /// The file names are decoded using the decoder.
    pub fn list_file_names(&self) -> Result<impl Iterator<Item = Result<String>>> {
        Ok(self
            .create_reader()?
            .list_file_names(self.get_decoding_fn()))
    }

    /// `read_file` reads the content of a file into the given output.
    /// It also returns the total number of bytes read.
    pub fn read_file<W: Write>(&self, file_name: &str, output: W) -> Result<usize> {
        self.create_reader()?
            .read_file(file_name, output, self.get_decoding_fn())
    }

    /// `read_file_by_block` reads the content of a file,
    /// and returns an iterator of the blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(
        &self,
        file_name: &str,
    ) -> Result<impl Iterator<Item = Result<Box<[u8]>>> + Send> {
        self.create_reader()?
            .read_file_by_block(file_name, self.get_decoding_fn())
    }

    /// `read_file_by_block` reads the content of a file,
    /// and returns an iterator of the blocks.
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

    fn get_decoding_fn(&self) -> Decoder {
        match self.decoder {
            Some(decoding_fn) => decoding_fn,
            None => Self::decode_utf8,
        }
    }

    fn decode_utf8(bytes: &[u8]) -> Option<Cow<'_, str>> {
        Some(String::from_utf8_lossy(bytes))
    }
}

// accessor
impl Archive {
    /// `path` returns the archive file path.
    pub fn path(&self) -> &Path {
        &self.file_path
    }
}
