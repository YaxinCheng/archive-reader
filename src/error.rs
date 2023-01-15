use crate::libarchive;
use std::ffi::CStr;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Extraction error: {0}")]
    Extraction(String),
    #[error("Archive path cannot be converted to utf8")]
    PathNotUtf8,
    #[error("Bytes cannot be decoded")]
    Encoding,
    #[error("Unknown error happened")]
    Unknown,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn analyze_result(
    result: std::os::raw::c_int,
    handle: *mut libarchive::archive,
) -> Result<()> {
    match result {
        libarchive::ARCHIVE_OK | libarchive::ARCHIVE_WARN => Ok(()),
        _ => unsafe {
            let error_string = libarchive::archive_error_string(handle);
            if !error_string.is_null() {
                return Err(Error::Extraction(
                    CStr::from_ptr(error_string).to_string_lossy().to_string(),
                ));
            }
            let error_code = libarchive::archive_errno(handle);
            if error_code != 0 {
                Err(std::io::Error::from_raw_os_error(error_code).into())
            } else {
                Err(Error::Unknown)
            }
        },
    }
}

pub(crate) fn path_does_not_exist(message: String) -> Error {
    Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, message))
}
