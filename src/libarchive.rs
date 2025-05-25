#![allow(non_camel_case_types)]

type la_int64_t = i64;
pub(crate) const ARCHIVE_EOF: i32 = 1;
pub(crate) const ARCHIVE_OK: i32 = 0;
pub(crate) const ARCHIVE_WARN: i32 = -20;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct archive {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct archive_entry {
    _unused: [u8; 0],
}
unsafe extern "C" {
    pub(crate) fn archive_entry_pathname(arg1: *mut archive_entry)
    -> *const ::std::os::raw::c_char;
    pub(crate) fn archive_errno(arg1: *mut archive) -> ::std::os::raw::c_int;
    pub(crate) fn archive_error_string(arg1: *mut archive) -> *const ::std::os::raw::c_char;
    pub(crate) fn archive_read_close(arg1: *mut archive) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_free(arg1: *mut archive) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_data_block(
        a: *mut archive,
        buff: *mut *const ::std::os::raw::c_void,
        size: *mut usize,
        offset: *mut la_int64_t,
    ) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_new() -> *mut archive;
    pub(crate) fn archive_read_next_header(
        arg1: *mut archive,
        arg2: *mut *mut archive_entry,
    ) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_open_filename(
        arg1: *mut archive,
        _filename: *const ::std::os::raw::c_char,
        _block_size: usize,
    ) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_support_filter_all(arg1: *mut archive) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_support_format_all(arg1: *mut archive) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_support_format_raw(arg1: *mut archive) -> ::std::os::raw::c_int;
    pub(crate) fn archive_read_add_passphrase(
        archive: *mut archive,
        passphrase: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
