use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum TftpEncoding {
    Netascii,
    Octet,
}

impl TryFrom<String> for TftpEncoding {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "netascii" => Ok(TftpEncoding::Netascii),
            "octet" => Ok(TftpEncoding::Octet),
            _ => Err(String::from("invalid encoding").into()),
        }
    }
}

/// Encodes the provided data slice according to the TFTP encoding rules.
pub fn tftp_encode(
    data: &[u8],
    encoding: TftpEncoding
) -> Vec<u8> {
    match encoding {
        TftpEncoding::Octet => data.to_vec(),
        TftpEncoding::Netascii => {
            let mut encoded = Vec::with_capacity(data.len() + data.len() / 10);
            for &b in data {
                match b {
                    b'\n' => { encoded.push(b'\r'); encoded.push(b'\n'); }
                    b'\r' => { encoded.push(b'\r'); encoded.push(b'\0'); }
                    _ => encoded.push(b),
                }
            }
            encoded
        }
    }
}
