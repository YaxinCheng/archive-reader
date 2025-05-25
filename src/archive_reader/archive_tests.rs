use crate::error::{Error, Result};
use crate::Archive;

const fn zip_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.zip")
}

const fn seven_z_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.7z")
}

const fn rar_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.rar")
}

const fn encrypted_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/encrypted.zip")
}

// 7z can encrypt even the file names.
const fn encrypted_7z() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/encrypted.7z")
}

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

#[test]
fn test_read_dir() -> Result<()> {
    test_read_file_to_bytes(zip_archive(), "content/", b"")
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

#[test]
#[cfg(not(feature = "lending_iter"))]
fn test_file_names_from_entries() -> Result<()> {
    let mut names = vec![];
    Archive::open(zip_archive()).entries(|entry| {
        let file_name = entry.file_name()?.to_string();
        names.push(file_name);
        Ok(())
    })?;
    let expected = [
        "content/",
        "content/first",
        "content/third",
        "content/nested/",
        "content/nested/second",
    ];
    assert_eq!(names, expected);
    Ok(())
}

#[test]
#[cfg(feature = "lending_iter")]
fn test_file_names_from_entries() -> Result<()> {
    use crate::LendingIterator;

    let mut names = vec![];
    let mut entries = Archive::open(zip_archive()).entries()?;
    while let Some(entry) = entries.next() {
        let file_name = entry?.file_name()?.to_string();
        names.push(file_name);
    }
    let expected = [
        "content/",
        "content/first",
        "content/third",
        "content/nested/",
        "content/nested/second",
    ];
    assert_eq!(names, expected);
    Ok(())
}

#[test]
#[cfg(not(feature = "lending_iter"))]
fn test_file_content_from_entries() -> Result<()> {
    let mut all_content = vec![];
    Archive::open(zip_archive()).entries(|entry| {
        let mut content = Vec::<u8>::new();
        let blocks = entry.read_file_by_block();
        for block in blocks {
            content.extend(block?.iter())
        }
        all_content.push(content);
        Ok(())
    })?;
    let expected: Vec<&[u8]> = vec![b"", b"first\n", b"third\n", b"", b"second\n"];
    assert_eq!(expected, all_content);
    Ok(())
}

#[test]
#[cfg(feature = "lending_iter")]
fn test_file_content_from_entries() -> Result<()> {
    use crate::LendingIterator;

    let mut all_content = vec![];
    let mut entries = Archive::open(zip_archive()).entries()?;
    while let Some(entry) = entries.next() {
        let mut content = Vec::<u8>::new();
        let mut blocks = entry?.read_file_by_block();
        while let Some(block) = blocks.next() {
            content.extend(block?.iter())
        }
        all_content.push(content);
    }
    let expected: Vec<&[u8]> = vec![b"", b"first\n", b"third\n", b"", b"second\n"];
    assert_eq!(expected, all_content);
    Ok(())
}

#[test]
#[cfg(not(feature = "lending_iter"))]
fn test_entry_name_reproducible() -> Result<()> {
    Archive::open(zip_archive()).entries(|entry| {
        assert_eq!(entry.file_name()?, entry.file_name()?);
        Ok(())
    })?;
    Ok(())
}

#[test]
#[cfg(feature = "lending_iter")]
fn test_entry_name_reproducible() -> Result<()> {
    use crate::LendingIterator;
    let mut entries = Archive::open(zip_archive()).entries()?;
    while let Some(entry) = entries.next() {
        let entry = entry?;
        assert_eq!(entry.file_name()?, entry.file_name()?);
    }
    Ok(())
}

#[test]
fn test_read_file_names_from_encrypted_archive_success() -> Result<()> {
    let file_names = Archive::open(encrypted_archive())
        .list_file_names()?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(file_names, ["encrypted"]);
    Ok(())
}

#[test]
fn test_read_encrypted_archive_failed_without_password() -> Result<()> {
    let mut file_content = vec![];
    let read_result = Archive::open(encrypted_archive()).read_file("encrypted", &mut file_content);
    let error_msg = match read_result {
        Err(Error::Extraction(msg)) => msg,
        unexpected => panic!("unexpected result type: {unexpected:?}"),
    };
    assert_eq!(error_msg, "Passphrase required for this entry");
    Ok(())
}

#[test]
fn test_read_encrypted_archive_failed_with_empty_password() -> Result<()> {
    let mut file_content = vec![];
    let read_result = Archive::open(encrypted_archive())
        .try_password("")
        .read_file("encrypted", &mut file_content);
    let error_msg = match read_result {
        Err(Error::Extraction(msg)) => msg,
        unexpected => panic!("unexpected result type: {unexpected:?}"),
    };
    assert_eq!(error_msg, "Empty passphrase is unacceptable");
    Ok(())
}

#[test]
fn test_read_encrypted_archive_failed_wrong_password() -> Result<()> {
    let mut file_content = vec![];
    let read_result = Archive::open(encrypted_archive())
        .try_password("wrong")
        .read_file("encrypted", &mut file_content);
    let error_msg = match read_result {
        Err(Error::Extraction(msg)) => msg,
        unexpected => panic!("unexpected result type: {unexpected:?}"),
    };
    assert_eq!(error_msg, "Incorrect passphrase");
    Ok(())
}

#[test]
fn test_read_encrypted_archive_success() -> Result<()> {
    let mut file_content = vec![];
    Archive::open(encrypted_archive())
        .try_password("password")
        .read_file("encrypted", &mut file_content)?;
    assert_eq!(file_content, b"encrypted\n");
    Ok(())
}

#[test]
fn test_read_encrypted_archive_success_with_multiple_password() -> Result<()> {
    let mut file_content = vec![];
    Archive::open(encrypted_archive())
        .try_password("password")
        .try_password("wrong")
        .try_password("wrong2")
        .read_file("encrypted", &mut file_content)?;
    assert_eq!(file_content, b"encrypted\n");
    Ok(())
}

#[test]
fn test_read_file_names_from_encrypted_7z_failed() -> Result<()> {
    let file_names = Archive::open(encrypted_7z())
        .try_password("password")
        .list_file_names()?
        .collect::<Result<Vec<_>>>();
    let error_msg = match file_names {
        Err(Error::Extraction(msg)) => msg,
        unexpected => panic!("unexpected result type: {unexpected:?}"),
    };
    assert_eq!(
        error_msg,
        "The archive header is encrypted, but currently not supported"
    );
    Ok(())
}
