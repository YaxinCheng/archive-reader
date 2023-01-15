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
