use super::*;
use crate::error::Result;

const fn zip_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.zip")
}

const fn seven_z_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.7z")
}

const fn rar_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.rar")
}

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
    let archive = ArchiveReader::open(path)?;
    let mut file_names = archive.list_file_names().collect::<Result<Vec<_>>>()?;
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

fn test_read_file_to_bytes(archive_path: &str, file_path: &str, expected: &[u8]) -> Result<()> {
    let archive = ArchiveReader::open(archive_path)?;
    let bytes = archive.read_file_to_bytes(file_path)?;
    assert_eq!(bytes, expected);
    Ok(())
}

#[test]
fn test_read_by_blocks() -> Result<()> {
    let archive = ArchiveReader::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_resources/large.zip"
    ))?;
    let mut num_of_blocks = 0_usize;
    let mut total_size = 0_usize;
    for block in archive.read_file_by_block("large.txt")? {
        let block = block?;
        num_of_blocks += 1;
        total_size += block.len();
    }
    assert!(num_of_blocks > 1);
    let expected_size = 819201;
    assert_eq!(total_size, expected_size);
    Ok(())
}
