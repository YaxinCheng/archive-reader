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
    let file_names = Archive::open(zip_archive())
        .list_file_names()?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(
        file_names,
        [
            "content/",
            "content/first",
            "content/third",
            "content/nested/",
            "content/nested/second",
        ]
    );
    Ok(())
}

#[test]
fn test_list_7z_file_names() -> Result<()> {
    let file_names = Archive::open(seven_z_archive())
        .list_file_names()?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(
        file_names,
        [
            "content/",
            "content/nested/",
            "content/first",
            "content/nested/second",
            "content/third",
        ]
    );
    Ok(())
}

#[test]
fn test_list_rar_file_names() -> Result<()> {
    let file_names = Archive::open(rar_archive())
        .list_file_names()?
        .collect::<Result<Vec<_>>>()?;
    assert_eq!(
        file_names,
        [
            "content/first",
            "content/third",
            "content/nested/second",
            "content/nested",
            "content",
        ]
    );
    Ok(())
}

#[test]
fn test_read_zip() -> Result<()> {
    let mut output = vec![];
    let _ = Archive::open(zip_archive()).read_file("content/nested/second", &mut output)?;
    assert_eq!(output, b"second\n");
    Ok(())
}

#[test]
fn test_read_7z() -> Result<()> {
    let mut output = vec![];
    let _ = Archive::open(seven_z_archive()).read_file("content/nested/second", &mut output)?;
    assert_eq!(output, b"second\n");
    Ok(())
}

#[test]
fn test_read_rar() -> Result<()> {
    let mut output = vec![];
    let _ = Archive::open(rar_archive()).read_file("content/nested/second", &mut output)?;
    assert_eq!(output, b"second\n");
    Ok(())
}

#[test]
fn test_read_non_existing_file() -> Result<()> {
    let mut output = vec![];
    let read_result = Archive::open(zip_archive()).read_file("not_existed", &mut output);
    assert_eq!(
        read_result,
        Err(Error::Io(std::io::ErrorKind::NotFound.into()))
    );
    Ok(())
}

#[test]
fn test_empty_file() -> Result<()> {
    let mut output = vec![];
    let _ = Archive::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test_resources/empty.zip"
    ))
    .read_file("empty", &mut output)?;
    assert_eq!(output, b"");
    Ok(())
}

#[test]
fn test_read_dir() -> Result<()> {
    let mut output = vec![];
    let _ = Archive::open(zip_archive()).read_file("content/", &mut output)?;
    assert_eq!(output, b"");
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
    assert_eq!(
        names,
        [
            "content/",
            "content/first",
            "content/third",
            "content/nested/",
            "content/nested/second",
        ]
    );
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
    assert_eq!(
        names,
        [
            "content/",
            "content/first",
            "content/third",
            "content/nested/",
            "content/nested/second",
        ]
    );
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
    assert_eq!(
        read_result,
        Err(Error::Extraction(
            "Passphrase required for this entry".into()
        ))
    );
    Ok(())
}

#[test]
fn test_read_encrypted_archive_failed_with_empty_password() -> Result<()> {
    let mut file_content = vec![];
    let read_result = Archive::open(encrypted_archive())
        .try_password("")
        .read_file("encrypted", &mut file_content);
    assert_eq!(
        read_result,
        Err(Error::Extraction("Empty passphrase is unacceptable".into()))
    );
    Ok(())
}

#[test]
fn test_read_encrypted_archive_failed_wrong_password() -> Result<()> {
    let mut file_content = vec![];
    let read_result = Archive::open(encrypted_archive())
        .try_password("wrong")
        .read_file("encrypted", &mut file_content);
    assert_eq!(
        read_result,
        Err(Error::Extraction("Incorrect passphrase".into()))
    );
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
    assert_eq!(
        file_names,
        Err(Error::Extraction(
            "The archive header is encrypted, but currently not supported".into()
        ))
    );
    Ok(())
}
