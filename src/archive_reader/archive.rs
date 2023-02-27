use crate::archive_reader::blocks::{BlockReader, BlockReaderBorrowed};
use crate::archive_reader::entries::Entries;
use crate::error::Result;
use crate::Entry;
use log::info;
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
}

// Consumers
impl Archive {
    /// `list_file_names` return an iterator of file names extracted from the archive.
    /// The file names are in bytes,
    /// and users can choose their own decoder to decode the bytes into string.
    pub fn list_file_names(&self) -> Result<impl Iterator<Item = Result<bytes::Bytes>> + Send> {
        info!("Archive::list_file_names()");
        self.list_entries().map(|entries| entries.file_names())
    }

    /// `read_file` reads the content of a file into the given output.
    /// It also returns the total number of bytes read.
    pub fn read_file<W: Write, P>(&self, predicate: P, mut output: W) -> Result<usize>
    where
        P: Fn(&[u8]) -> bool,
    {
        info!(r#"Archive::read_file(predicate: _, output: _)"#);
        let mut entries = self.list_entries()?;
        entries.find_entry_by_name(predicate)?;
        let mut blocks = BlockReaderBorrowed::from(&entries);
        let mut written = 0;
        while let Some(block) = crate::LendingIterator::next(&mut blocks) {
            let block = block?;
            written = block.len();
            output.write_all(block)?;
        }
        Ok(written)
    }

    /// `read_file_by_block` reads the content of a file,
    /// and returns an iterator of the blocks.
    #[cfg(not(feature = "lending_iter"))]
    pub fn read_file_by_block<P>(
        &self,
        predicate: P,
    ) -> Result<impl Iterator<Item = Result<bytes::Bytes>> + Send>
    where
        P: Fn(&[u8]) -> bool,
    {
        info!(r#"Archive::read_file_by_block(predicate: _)"#);
        let mut entries = self.list_entries()?;
        entries.find_entry_by_name(predicate)?;
        Ok(BlockReader::new(entries))
    }

    /// `read_file_by_block` reads the content of a file,
    /// and returns an iterator of the blocks.
    #[cfg(feature = "lending_iter")]
    pub fn read_file_by_block<P>(
        &self,
        predicate: P,
    ) -> Result<impl for<'a> crate::LendingIterator<Item<'a> = Result<&'a [u8]>> + Send>
    where
        P: Fn(&[u8]) -> bool,
    {
        info!(r#"Archive::read_file_by_block(predicate: _)"#);
        let mut entries = self.list_entries()?;
        entries.find_entry_by_name(predicate)?;
        Ok(BlockReader::new(entries))
    }

    /// `entries` iterates through each file / dir in the archive,
    /// and passes the mutable references of the entries to the process closure.
    /// Using the functions provided on the `Entry` object,
    /// one can obtain two things from each entry:
    ///   1. name
    ///   2. content
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
    /// one can obtain two things from each entry:
    ///   1. name
    ///   2. content
    #[cfg(feature = "lending_iter")]
    pub fn entries(
        &self,
    ) -> Result<impl for<'a> crate::LendingIterator<Item<'a> = Result<Entry<'a>>>> {
        info!(r#"Archive::entries()"#);
        self.list_entries()
    }
}

// accessor
impl Archive {
    fn list_entries(&self) -> Result<Entries> {
        Entries::open(&self.file_path, self.block_size)
    }
    /// `path` returns the archive file path.
    pub fn path(&self) -> &Path {
        &self.file_path
    }
}
