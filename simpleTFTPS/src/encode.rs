use std::ffi::{CString};
use std::os::raw::c_char;
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TftpEncoding {
    Netascii,
    Octet,
}
impl TryFrom<CString> for TftpEncoding {
    type Error = Error;
    fn try_from(value: CString) -> Result<Self, Self::Error> {
        match value.to_str()?.to_lowercase().as_str() {
            "netascii" => Ok(TftpEncoding::Netascii),
            "octet" => Ok(TftpEncoding::Octet),
            _ => Err(String::from("invalid encoding").into()),
        }
    }
}

/// Takes OWNERSHIP of a C string, encodes it, and returns a new buffer.
/// DANGER: Rust will free `c_str_ptr` at the end of this function.
pub fn tftp_encode(
    c_str_ptr: *mut c_char, // MUST be *mut because Rust takes ownership to free it
    encoding: TftpEncoding
) -> Vec<u8> {
    if c_str_ptr.is_null() {
        return Vec::new();
    }

    // 1. Take ownership of the pointer.
    // SAFETY: We assume this pointer was created by Rust's allocator,
    // or that C and Rust share the exact same allocator.
    let owned_cstring = unsafe { CString::from_raw(c_str_ptr) };

    // 2. Access the bytes safely without copying
    let bytes = owned_cstring.as_bytes();

    let mut vec = match encoding {
        TftpEncoding::Octet => bytes.to_vec(),
        TftpEncoding::Netascii => {
            let mut encoded = Vec::with_capacity(bytes.len() + bytes.len() / 10);
            for &b in bytes {
                match b {
                    b'\n' => { encoded.push(b'\r'); encoded.push(b'\n'); }
                    b'\r' => { encoded.push(b'\r'); encoded.push(b'\0'); }
                    _ => encoded.push(b),
                }
            }
            encoded
        }
    };

    // 3. Prepare the new buffer to hand back to C
    vec.shrink_to_fit();
    vec
}