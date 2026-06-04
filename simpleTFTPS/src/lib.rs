
use std::ffi::{c_char, CStr, CString};
pub mod error;
pub mod encode;

use std::net::{SocketAddr, UdpSocket};
use std::thread;
use error::Error;
use encode::TftpEncoding;
use crate::encode::tftp_encode;


// to run from rust natively
pub unsafe fn run_rs<F1, F2>(
    callback_get: F1,
    callback_put: F2,
    address: String
) -> Result<(), Error>
where
    F1: Fn(String) -> Option<*mut c_char> + Send + Sync + 'static + Copy,
    F2: Fn(String) -> Option<*mut c_char> + Send + Sync + 'static + Copy,
{

    let sock = UdpSocket::bind(address.as_str())?;
    loop {
        let mut buf = [0u8; 1024];
        let (_len, addr) = sock.recv_from(&mut buf)?;
        thread::spawn(move || {
            let _res:Result<(),Error> = || -> Result<(), Error> {

                //child process
                let req_num = ((buf[0] as u16) << 8) + (buf[1] as u16);
                let req: Request = Request::try_from(req_num)?;

                let file_name_cstr = CStr::from_bytes_until_nul(&buf[2..])
                    .map_err(|_| Error::Str("Invalid or missing filename".to_string()))?;
                let file_name = file_name_cstr.to_owned().into_string()?;

                let encoding_start = 2 + file_name.as_bytes().len() + 1;

                let raw_encoding_cstr = CStr::from_bytes_until_nul(&buf[encoding_start..])
                    .map_err(|_| Error::Str("Invalid or missing encoding".to_string()))?;
                let encoding = TftpEncoding::try_from(raw_encoding_cstr.to_owned())?;

                let sock = UdpSocket::bind("127.0.0.1:0")?; // open socket on a new emply port for the current Request

                match req {
                    Request::RRQ => {
                        let data_ptr = callback_get(file_name).ok_or_else(|| ErrorID::FileNotFound("file not found".to_string()));
                        if data_ptr.is_err() {
                            send_error(data_ptr.clone().err().unwrap(), sock, addr)?;
                            return Err(Error::from(<ErrorID as Into<String>>::into(data_ptr.err().unwrap())))
                        }
                        let data = tftp_encode(data_ptr.ok().unwrap(), encoding);
                        let packet_num = (data.len() as f32 / 512.0).ceil() as u16;
                        
                        for i in 1..=packet_num {
                            let mut packet = Vec::with_capacity(516);
                            packet.extend_from_slice(&(Request::DATA as u16).to_be_bytes());
                            packet.extend_from_slice(&i.to_be_bytes());
                            let start = ((i as usize) - 1) * 512;
                            let end = (i as usize) * 512;
                            if end > data.len() {
                                packet.extend_from_slice(&data[start..]);
                            }else{
                                packet.extend_from_slice(&data[start..end]);
                            }
                            sock.send_to(packet.as_slice(), addr)?;
                            
                            let mut ack_buf = [0u8; 1024];
                            let (_ack_len, _ack_addr) = sock.recv_from(&mut ack_buf)?;
                            let ack_req_num = ((ack_buf[0] as u16) << 8) + (ack_buf[1] as u16);
                            let ack_req: Request = Request::try_from(ack_req_num)?;
                            match ack_req {
                                Request::ACK => {
                                    let ack_block = ((ack_buf[2] as u16) << 8) + (ack_buf[3] as u16);
                                    if ack_block != i {
                                        return Err(Error::Str(format!("invalid ACK block: expected {}, got {}", i, ack_block)));
                                    } else {
                                        Ok(())
                                    }
                                },
                                Request::ERR => {
                                    Err(
                                        Error::Str(format!("error:{}", CStr::from_bytes_with_nul(&ack_buf[2..]).unwrap().to_str().unwrap()))
                                    )
                                }
                                _ => Err(Error::Str("expected ACK".to_string())),
                            }?;
                        }
                        
                        // If data.len() is a multiple of 512, we must send an empty packet
                        if data.len() > 0 && data.len() % 512 == 0 {
                             let mut packet = Vec::with_capacity(4);
                             packet.extend_from_slice(&(Request::DATA as u16).to_be_bytes());
                             packet.extend_from_slice(&((packet_num + 1) as u16).to_be_bytes());
                             sock.send_to(packet.as_slice(), addr)?;
                             let mut ack_buf = [0u8; 1024];
                             let _ = sock.recv_from(&mut ack_buf);
                        }
                        if data.len() == 0 {
                             let mut packet = Vec::with_capacity(4);
                             packet.extend_from_slice(&(Request::DATA as u16).to_be_bytes());
                             packet.extend_from_slice(&(1u16).to_be_bytes());
                             sock.send_to(packet.as_slice(), addr)?;
                             let mut ack_buf = [0u8; 1024];
                             let _ = sock.recv_from(&mut ack_buf);
                        }


                        Ok(())
                    }
                    Request::WRQ => {
                        let _ = callback_put(file_name).ok_or("file not found");
                        Ok(())
                    }
                    _ => Err(Error::Str("invalid request".to_string())),
                }?;


                Ok(())
            }();
        });
    }
}
pub fn send_error(err:ErrorID, sock:UdpSocket,addr:SocketAddr)->Result<(),Error>{
    let _err_num = err.get_id();

    let len = <ErrorID as Into<String>>::into(err.clone()).len() + 5;
    let mut packet = Vec::with_capacity(len);
    packet.extend_from_slice(&(Request::ERR as u16).to_be_bytes());
    packet.extend_from_slice(CString::new(<ErrorID as Into<String>>::into(err).as_str())?.to_bytes_with_nul());
    packet.extend_from_slice(&0u8.to_be_bytes());
    sock.send_to(packet.as_slice(),addr)?;
    Ok(())
}

//to run from c
#[unsafe(no_mangle)]
pub unsafe extern "C" fn run(
    callback_get: unsafe extern "C" fn(file: *mut c_char) -> *mut c_char,
    callback_put: unsafe extern "C" fn(file: *mut c_char) -> *mut c_char,
    address: *const c_char
) {
    if address.is_null() {
        return;
    }

    let addr_cstr = CStr::from_ptr(address);
    let addr_str = addr_cstr.to_str().unwrap_or("127.0.0.1:69").to_string();

    let wrapped_get = move |file: String| {
        let c_str = CString::new(file).unwrap();
        let ptr = unsafe{callback_get(c_str.as_ptr() as *mut c_char)};

        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    };

    let wrapped_put = move |file: String| {
        let c_str = CString::new(file).unwrap();
        let ptr = unsafe{callback_put(c_str.as_ptr() as *mut c_char)};
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    };

    let _ = run_rs(wrapped_get, wrapped_put, addr_str);
}

#[repr(u16)]
pub enum Request {
    RRQ = 1, // read Request
    WRQ = 2, // write Request
    DATA = 3,
    ACK = 4,
    ERR = 5,
}
impl TryFrom<u16> for Request{
    type Error = String;

    fn try_from(value:u16)->Result<Self,Self::Error>{
        match value {
            1 => Ok(Request::RRQ),
            2 => Ok(Request::WRQ),
            3 => Ok(Request::DATA),
            4 => Ok(Request::ACK),
            5 => Ok(Request::ERR),
            _ => Err(format!("invalid request num:{}",value)),
        }
    }
}
#[derive(Debug,Clone)]
pub enum ErrorID{
    NotDefined(String),
    FileNotFound(String),
    AccessViolation(String),
    DiskFull(String),
    IllegalOperation(String),
    UnknownTransferId(String),
    FileAlreadyExists(String),
    NoSuchUser(String),
}
impl ErrorID{
    pub fn get_id(&self)->u16{
        match self{
            ErrorID::NotDefined(_)=>0,
            ErrorID::FileNotFound(_)=>1,
            ErrorID::AccessViolation(_)=>2,
            ErrorID::DiskFull(_)=>3,
            ErrorID::IllegalOperation(_)=>4,
            ErrorID::UnknownTransferId(_)=>5,
            ErrorID::FileAlreadyExists(_)=>6,
            ErrorID::NoSuchUser(_)=>7,
        }
    }
}impl Into<String> for ErrorID{
    fn into(self) -> String {
        match self {
            ErrorID::NotDefined(s)|
            ErrorID::FileNotFound(s)|
            ErrorID::AccessViolation(s)|
            ErrorID::DiskFull(s)|
            ErrorID::IllegalOperation(s)|
            ErrorID::UnknownTransferId(s)|
            ErrorID::FileAlreadyExists(s)|
            ErrorID::NoSuchUser(s)=>{
                s
            }
        }
    }
}