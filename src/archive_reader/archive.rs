use crate::archive_reader::blocks::{BlockReader, BlockReaderBorrowed};
use crate::archive_reader::entries::Entries;
use crate::error::Result;
use crate::{Decoder, Entry};
use log::info;
use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_BLOCK_SIZE: usize = 1024 * 1024;

/// `Archive` represents an archive file which can be processed.
pub struct Archive {
    /// `block_size` is a size that will be used to break down content into blocks.
    /// The blocks read from the archive are not exactly the size of the `block_size`,
    /// due to compression and other factors.
    /// Increasing the size of this variable can make the reader read more content
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
    /// It handles the path lazily. So no error will occur until operations are operated on
    /// the archive.
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        fn open_with_path(path: &Path) -> Archive {
            Archive {
                block_size: DEFAULT_BLOCK_SIZE,
                file_path: path.into(),
                decoder: None,
            }
        }
        open_with_path(path.as_ref())
    }

    /// `block_size` sets the size limit for every block reading from the archive.
    /// The block size is represented in bytes.
    ///
    /// # Note:
    /// The content from archives is read block by block.
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
    /// The default decoder converts the bytes into a UTF-8 encoded string.
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
    pub fn list_file_names(&self) -> Result<impl Iterator<Item = Result<String>> + Send> {
        info!("Archive::list_file_names()");
        self.list_entries().map(Entries::file_names)
    }

    /// `read_file` reads the content of a file into the given output.
    /// It also returns the total number of bytes read.
    pub fn read_file<W: Write>(&self, file_name: &str, mut output: W) -> Result<usize> {
        info!(r#"Archive::read_file(file_name: "{file_name}", output: _)"#);
        let mut entries = self.list_entries()?;
        entries.find_entry_by_name(file_name)?;
        let mut blocks = BlockReaderBorrowed::from(&entries);
        let mut written = 0;
        while let Some(block) = crate::LendingIterator::next(&mut blocks) {
            let block = block?;
            written = block.len();
            output.write_all(block)?;
        }
        Ok(written)
    }

    /// `read_file_by_block` reads the content of a file
    /// and returns an iterator of the blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block(
        &self,
        file_name: &str,
    ) -> Result<impl Iterator<Item = Result<Box<[u8]>>> + Send + use<>> {
        info!(r#"Archive::read_file_by_block(file_name: "{file_name}")"#);
        let mut entries = self.list_entries()?;
        entries.find_entry_by_name(file_name)?;
        Ok(BlockReader::new(entries))
    }

    /// `read_file_by_block` reads the content of a file
    /// and returns an iterator of the blocks.
    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block(
        &self,
        file_name: &str,
    ) -> Result<impl for<'a> crate::LendingIterator<Item<'a> = Result<&'a [u8]>> + Send> {
        info!(r#"Archive::read_file_by_block(file_name: "{file_name}")"#);
        let mut entries = self.list_entries()?;
        entries.find_entry_by_name(file_name)?;
        Ok(BlockReader::new(entries))
    }

    /// `entries` iterates through each file / dir in the archive
    /// and passes the mutable references of the entries to the process closure.
    /// Using the functions provided on the `Entry` object,
    /// one can get two things from each entry:
    ///   1. Name
    ///   2. Content
    #[cfg(not(feature = "lending_iter"))]
    pub fn entries<F>(&self, mut process: F) -> Result<()>
    where
        F: FnMut(Entry) -> Result<()>,
    {
        info!(r#"Archive::entries(process: _)"#);
        let mut entries = self.list_entries()?;
        while let Some(entry) = entries.next() {
            process(entry?)?
        }
        Ok(())
    }

    /// `entries` returns a lending iterator of `Entry`s.
    /// Each `Entry` represents a file / dir in an archive.
    /// Using the functions provided on the `Entry` object,
    /// one can get two things from each entry:
    ///   1. Name
    ///   2. Content
    #[cfg(feature = "lending_iter")]
    pub fn entries(
        &self,
    ) -> Result<impl for<'a> crate::LendingIterator<Item<'a> = Result<Entry<'a>>>> {
        info!(r#"Archive::entries()"#);
        self.list_entries()
    }
}

// util functions
impl Archive {
    fn list_entries(&self) -> Result<Entries> {
        Entries::open(&self.file_path, self.block_size, self.get_decoding_fn())
    }

    fn get_decoding_fn(&self) -> Decoder {
        match self.decoder {
            Some(decoding_fn) => decoding_fn,
            None => Self::decode_utf8,
        }
    }

    fn decode_utf8(bytes: &[u8]) -> Option<Cow<'_, str>> {
        std::str::from_utf8(bytes).map(Cow::Borrowed).ok()
    }
}

// accessor
impl Archive {
    /// `path` returns the archive file path.
    pub fn path(&self) -> &Path {
        &self.file_path
    }
}
