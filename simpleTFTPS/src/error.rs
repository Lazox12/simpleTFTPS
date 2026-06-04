use std::ffi::{IntoStringError, NulError};
use std::io::Error as IoError;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error{
    InvalidRequest,
    Io(IoError),
    Str(String),
    Utf8(Utf8Error),
    FromUtf8(FromUtf8Error),
    Nul(NulError),
    IntoString(IntoStringError),
}
impl From<IoError> for Error{
    fn from(err:IoError)->Self{
        Error::Io(err)
    }
}
impl From<String> for Error{
    fn from(err:String)->Self{
        Error::Str(err)
    }
}
impl From<Utf8Error> for Error{
    fn from(err:Utf8Error)->Self{
        Error::Utf8(err)
    }
}
impl From<NulError> for Error{
    fn from(err:NulError)->Self{
        Error::Nul(err)
    }
}
impl From<FromUtf8Error> for Error{
    fn from(err:FromUtf8Error)->Self{
        Error::FromUtf8(err)
    }
}
impl From<IntoStringError> for Error{
    fn from(value: IntoStringError) -> Self {
        Error::IntoString(value)
    }
}
