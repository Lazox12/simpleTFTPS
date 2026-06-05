extern crate core;

use core::time;
use std::ffi::{c_char, CStr, CString};
pub mod error;
pub mod encode;

use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::thread;
use std::thread::sleep;
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
    F1: Fn(String) -> Option<Vec<u8>> + Send + Sync + 'static + Copy,
    F2: Fn(String) -> Option<Vec<u8>> + Send + Sync + 'static + Copy,
{

    let sock = UdpSocket::bind(address.as_str())?;
    loop {
        let mut buf = [0u8; 1024];
        let (_len, addr) = sock.recv_from(&mut buf)?;
        thread::spawn(move || {
            let res:Result<(),Error> = || -> Result<(), Error> {

                //child process
                let req_num = ((buf[0] as u16) << 8) + (buf[1] as u16);
                let req: Request = Request::try_from(req_num)?;

                let file_name_cstr = CStr::from_bytes_until_nul(&buf[2..])
                    .map_err(|_| Error::Str("Invalid or missing filename".to_string()))?;
                let file_name = file_name_cstr.to_owned().into_string()?;

                let encoding_start = 2 + file_name.as_bytes().len() + 1;

                let raw_encoding_cstr = CStr::from_bytes_until_nul(&buf[encoding_start..])
                    .map_err(|_| Error::Str("Invalid or missing encoding".to_string()))?;
                let encoding = TftpEncoding::try_from(raw_encoding_cstr.to_owned().to_str().unwrap_or("octet").to_string())?;

                let options_start = encoding_start + raw_encoding_cstr.to_bytes().len()+1;
                let options_raw = &buf[options_start..];
                let mut options = vec![];
                let mut i = 0usize;
                loop{
                    let s = CStr::from_bytes_until_nul(&options_raw[i..]).map_err(|_| Error::Str("Invalid or missing option".to_string()))?.to_str()?;
                    i+=s.len();
                    i+=1;
                    if(s.len()==0){
                        break;
                    }
                    options.append(&mut vec![s.to_string()]);
                }

                let options = TftpOption::from_vec(options)?;
                let mut accepted_options = vec![];
                let sock = UdpSocket::bind(SocketAddr::new(IpAddr::from([0,0,0,0]),0))?; // open socket on a new emply port for the current Request

                match req {
                    Request::RRQ => {
                        let packet_len=options
                            .iter()
                            .find(|x| {x.name==TftpOptionType::blksize})
                            .map(|x|x.value)
                            .unwrap_or(512);

                        if packet_len!=512{
                            accepted_options.push(TftpOption{name:TftpOptionType::blksize,value:packet_len});
                        }

                        let data_raw = callback_get(file_name).ok_or_else(|| ErrorID::FileNotFound("file not found".to_string()));
                        if data_raw.is_err() {
                            send_error(data_raw.clone().err().unwrap(), sock, addr)?;
                            return Err(Error::from(<ErrorID as Into<String>>::into(data_raw.err().unwrap())))
                        }

                        let data = tftp_encode(&data_raw.unwrap(), encoding);
                        let packet_num = (data.len() as f32 / packet_len as f32).ceil() as u16;

                        if options.iter().find(|x| {x.name==TftpOptionType::tsize}).is_some(){ //if recieved transfer size request
                            accepted_options.push(TftpOption{name:TftpOptionType::tsize,value:data.len()});
                        }
                        send_oack(&sock,addr,accepted_options)?;

                        for i in 1..=packet_num {
                            let mut packet = Vec::with_capacity(packet_len+4);//2 for id, 2 for block number
                            packet.extend_from_slice(&(Request::DATA as u16).to_be_bytes());
                            packet.extend_from_slice(&i.to_be_bytes());
                            let start = ((i as usize) - 1) * packet_len;
                            let end = (i as usize) * packet_len;
                            if end > data.len() {
                                packet.extend_from_slice(&data[start..]);
                            }else{
                                packet.extend_from_slice(&data[start..end]);
                            }
                            //println!("sending packet {} of {} from port {}",i,packet_num, sock.local_addr().unwrap().port());
                            sock.send_to(packet.as_slice(), addr)?;
                            
                            let mut ack_buf = [0u8; 1024];
                            sock.recv(&mut ack_buf)?;
                            let r = parse_ack(&ack_buf)?;
                            if i!=r {
                                return Err(Error::Str(format!("ack failed for packet {}, got {}",i,r)));
                            }
                        }
                        
                        // If data.len() is a multiple of 512, we must send an empty packet
                        if data.len() > 0 && data.len() % packet_len == 0 {
                             let mut packet = Vec::with_capacity(4);
                             packet.extend_from_slice(&(Request::DATA as u16).to_be_bytes());
                             packet.extend_from_slice(&((packet_num + 1) as u16).to_be_bytes());
                             sock.send_to(packet.as_slice(), addr)?;
                             let mut ack_buf = [0u8; 1024];
                             let _ = sock.recv_from(&mut ack_buf);
                        }
                        /*
                        if data.len() == 0 {
                             let mut packet = Vec::with_capacity(4);
                             packet.extend_from_slice(&(Request::DATA as u16).to_be_bytes());
                             packet.extend_from_slice(&(1u16).to_be_bytes());
                             sock.send_to(packet.as_slice(), addr)?;
                             let mut ack_buf = [0u8; 1024];
                             let _ = sock.recv_from(&mut ack_buf);
                        }*/


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
            if res.is_err(){
                print!("{:?}",res.err().unwrap())
            }
        });
        //sleep(time::Duration::from_millis(100000000));
    }
}
pub fn parse_ack(buf: &[u8])->Result<u16,Error>{
    let ack_req_num = ((buf[0] as u16) << 8) + (buf[1] as u16);
    let ack_req: Request = Request::try_from(ack_req_num)?;
    match ack_req {
        Request::ACK => {
            let ack_block = ((buf[2] as u16) << 8) + (buf[3] as u16);
            Ok(ack_block)
        },
        Request::ERR => {
            Err(
                Error::Str(format!("error:{}", CStr::from_bytes_with_nul(&buf[3..]).or(Err("got error, failed to parse msg".to_string()))?.to_str()?))
            )
        }
        _ => Err(Error::Str("expected ACK".to_string())),
    }

}
pub fn send_error(err:ErrorID, sock:UdpSocket,addr:SocketAddr)->Result<(),Error>{
    let err_num = err.get_id();

    let mut packet = Vec::new();
    packet.extend_from_slice(&(Request::ERR as u16).to_be_bytes());
    packet.extend_from_slice(&err_num.to_be_bytes());
    packet.extend_from_slice(CString::new(<ErrorID as Into<String>>::into(err).as_str())?.to_bytes_with_nul());
    sock.send_to(packet.as_slice(),addr)?;
    Ok(())
}
pub fn send_oack(sock:&UdpSocket,addr:SocketAddr,options:Vec<TftpOption>)->Result<(),Error>{
    let mut packet = Vec::new();
    packet.extend_from_slice(&(Request::OACK as u16).to_be_bytes());
    for opt in options{
        packet.extend_from_slice(CString::new::<String>(opt.name.into())?.to_bytes_with_nul());
        packet.extend_from_slice(CString::new(opt.value.to_string())?.to_bytes_with_nul());
    }
    sock.send_to(packet.as_slice(),addr)?;
    let mut buf = [0u8; 1024];
    sock.recv(&mut buf)?;
    if parse_ack(&buf)? !=0{
        return Err(Error::Str("oack failed".to_string()));
    }
    Ok(())
}

unsafe extern "C" {
    fn free(ptr: *mut std::ffi::c_void);
}

//to run from c
#[unsafe(no_mangle)]
pub unsafe extern "C" fn run(
    callback_get: unsafe extern "C" fn(file: *const c_char, len: *mut usize) -> *mut c_char,
    callback_put: unsafe extern "C" fn(file: *const c_char, len: *mut usize) -> *mut c_char,
    address: *const c_char
) {
    if address.is_null() {
        return;
    }

    let addr_cstr = unsafe { CStr::from_ptr(address) };
    let addr_str = addr_cstr.to_str().unwrap_or("127.0.0.1:69").to_string();

    let wrapped_get = move |file: String| {
        let c_file = CString::new(file).unwrap();
        let mut len: usize = 0;
        let ptr = unsafe { callback_get(c_file.as_ptr(), &mut len) };
        if ptr.is_null() {
            None
        } else {
            let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
            unsafe { free(ptr as *mut _) };
            Some(data)
        }
    };

    let wrapped_put = move |file: String| {
        let c_file = CString::new(file).unwrap();
        let mut len: usize = 0;
        let ptr = unsafe { callback_put(c_file.as_ptr(), &mut len) };
        if ptr.is_null() {
            None
        } else {
            let data = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }.to_vec();
            unsafe { free(ptr as *mut _) };
            Some(data)
        }
    };

    let _ = unsafe { run_rs(wrapped_get, wrapped_put, addr_str) };
}

#[repr(u16)]
pub enum Request {
    RRQ = 1, // read Request
    WRQ = 2, // write Request
    DATA = 3,
    ACK = 4,
    ERR = 5,
    OACK = 6, //option ack
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
pub struct TftpOption{
    pub name:TftpOptionType,
    pub value:usize,
}
impl TftpOption{
    pub fn from_vec(data:Vec<String>)->Result<Vec<TftpOption>,Error>{

        data.chunks_exact(2).map(|x| {
            Ok(TftpOption{
                name:TftpOptionType::try_from(x[0].to_string())?,
                value:x[1].parse::<usize>().unwrap_or(0),
            })
        }).collect()
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug,Clone,Eq,PartialEq)]
pub enum TftpOptionType{
    tsize,
    blksize,
    timeout,
}
impl TryFrom<String> for TftpOptionType{
    type Error = String;
    fn try_from(value:String)->Result<Self,Self::Error>{
        match value.to_lowercase().as_str(){
            "tsize" => Ok(TftpOptionType::tsize),
            "blksize" => Ok(TftpOptionType::blksize),
            "timeout" => Ok(TftpOptionType::timeout),
            _ => Err(format!("invalid option type:{}",value)),
        }
    }
}impl Into<String> for TftpOptionType{
    fn into(self) -> String {
        match self {
            TftpOptionType::tsize => "tsize".to_string(),
            TftpOptionType::blksize => "blksize".to_string(),
            TftpOptionType::timeout => "timeout".to_string(),
        }
    }
}