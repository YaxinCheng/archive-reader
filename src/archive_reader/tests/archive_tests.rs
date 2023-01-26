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
        "content/first",
        "content/third",
        "content/nested/",
        "content/nested/second",
    ];
    test_list_file_names(seven_z_archive(), &expected)
}

#[test]
fn test_list_rar_file_names() -> Result<()> {
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
    let file_names = Archive::open(path)
        .list_file_names()?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(file_names, expected);
    Ok(())
}
