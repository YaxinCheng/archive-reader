use super::*;
use crate::error::Result;
use crate::Archive;

#[test]
fn test_list_zip_file_names() -> Result<()> {
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
fn test_list_7z_file_names() -> Result<()> {
    let expected = [
        "content/",
        "content/nested/",
        "content/first",
        "content/nested/second",
        "content/third",
    ];
    test_list_file_names(seven_z_archive(), &expected)
}

#[test]
fn test_list_rar_file_names() -> Result<()> {
    let expected = [
        "content/first",
        "content/third",
        "content/nested/second",
        "content/nested",
        "content",
    ];
    test_list_file_names(rar_archive(), &expected)
}

fn test_list_file_names(path: &str, expected: &[&str]) -> Result<()> {
    let file_names = Archive::open(path)
        .list_file_names()?
        .collect::<Result<Vec<_>>>()?;
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

fn test_read_file_to_bytes(archive_path: &str, content_path: &str, expected: &[u8]) -> Result<()> {
    let mut output = vec![];
    let _ = Archive::open(archive_path).read_file(content_path, &mut output)?;
    assert_eq!(output, expected);
    Ok(())
}

#[test]
fn test_read_by_blocks() -> Result<()> {
    #[cfg(feature = "lending_iter")]
    use crate::LendingIterator;

    let expected = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_resources/large.txt"
    ));
    let mut num_of_blocks = 0_usize;
    let mut bytes = Vec::new();
    let mut blocks = Archive::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_resources/large.zip"
    ))
    .block_size(1024)
    .read_file_by_block("large.txt")?;
    while let Some(block) = blocks.next() {
        let block = block?;
        num_of_blocks += 1;
        bytes.extend(block.iter());
    }
    assert!(num_of_blocks > 1);
    assert_eq!(expected, bytes.as_slice());
    Ok(())
}
