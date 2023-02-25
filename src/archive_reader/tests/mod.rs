mod archive_tests;

use std::borrow::Cow;

const fn zip_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.zip")
}

const fn seven_z_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.7z")
}

const fn rar_archive() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/test_resources/test.rar")
}

fn decode_utf8(bytes: &[u8]) -> Option<Cow<'_, str>> {
    Some(String::from_utf8_lossy(bytes))
}
