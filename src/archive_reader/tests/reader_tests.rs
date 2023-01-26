use super::*;
use crate::archive_reader::reader::ArchiveReader;
use crate::error::Result;

#[test]
fn test_list_file_names_zip() -> Result<()> {
    let expected = [
        "content/",
        "content/first",
        "content/third",
        "content/nested/",
        "content/nested/second",
    ];
    test_list_file_names(zip_archive(), &expected)
}

#[test]
fn test_list_file_names_7z() -> Result<()> {
    let expected = [
        "content/",
        "content/first",
        "content/third",
        "content/nested/",
        "content/nested/second",
    ];
    test_list_file_names(seven_z_archive(), &expected)
}

#[test]
fn test_list_file_names_rar() -> Result<()> {
    let expected = [
        "content",
        "content/first",
        "content/third",
        "content/nested",
        "content/nested/second",
    ];
    test_list_file_names(rar_archive(), &expected)
}

fn test_list_file_names(path: &str, expected: &[&str]) -> Result<()> {
    let archive = ArchiveReader::open(path, 1024)?;
    let mut file_names = archive
        .list_file_names(decode_utf8)
        .collect::<Result<Vec<_>>>()?;
    file_names.sort_by_key(|file_name| file_name.len());
    assert_eq!(file_names, expected);
    Ok(())
}

#[test]
fn test_read_zip() -> Result<()> {
    test_read_file_to_bytes(zip_archive(), "content/nested/second", b"second\n")
}

#[test]
fn test_read_7z() -> Result<()> {
    test_read_file_to_bytes(seven_z_archive(), "content/nested/second", b"second\n")
}

#[test]
fn test_read_rar() -> Result<()> {
    test_read_file_to_bytes(rar_archive(), "content/nested/second", b"second\n")
}

#[test]
#[should_panic]
fn test_read_non_existing_file() {
    test_read_file_to_bytes(zip_archive(), "not_existed", b"").unwrap()
}

#[test]
fn test_empty_file() -> Result<()> {
    let zip_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/empty.zip");
    test_read_file_to_bytes(zip_path, "empty", b"")
}

fn test_read_file_to_bytes(archive_path: &str, file_path: &str, expected: &[u8]) -> Result<()> {
    let archive = ArchiveReader::open(archive_path, 1024)?;
    let mut bytes = vec![];
    let _ = archive.read_file(file_path, &mut bytes, decode_utf8)?;
    assert_eq!(bytes, expected);
    Ok(())
}

#[test]
fn test_read_by_blocks() -> Result<()> {
    #[cfg(feature = "lending_iter")]
    use crate::LendingIterator;

    let archive = ArchiveReader::open(
        concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/large.zip"),
        1024,
    )?;
    let expected = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_resources/large.txt"
    ));
    let mut num_of_blocks = 0_usize;
    let mut bytes = Vec::new();
    let mut blocks = archive.read_file_by_block("large.txt", decode_utf8)?;
    while let Some(block) = blocks.next() {
        let block = block?;
        num_of_blocks += 1;
        bytes.extend(block.iter());
    }
    assert!(num_of_blocks > 1);
    assert_eq!(expected, bytes.as_slice());
    Ok(())
}

#[test]
fn test_empty_by_block() -> Result<()> {
    #[cfg(feature = "lending_iter")]
    use crate::LendingIterator;

    let archive = ArchiveReader::open(
        concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/empty.zip"),
        1024,
    )?;
    let mut blocks = archive.read_file_by_block("empty", decode_utf8)?;
    let mut number_of_blocks = 0;
    let mut bytes = Vec::<u8>::new();
    while let Some(block) = blocks.next() {
        let block = block?;
        number_of_blocks += 1;
        bytes.extend(block.iter());
    }
    assert_eq!(number_of_blocks, 0);
    assert_eq!(bytes, &[]);
    Ok(())
}
