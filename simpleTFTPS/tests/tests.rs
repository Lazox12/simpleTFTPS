
#[cfg(test)]
mod tests {

    use std::ffi::{c_char, CString};
    use simpleTFTPS::{run_rs};
    use simpleTFTPS::error::Error;
    use std::process::Command;
    use std::thread;

    #[test]
    fn test_read()->Result<(),Error> {
        fn callback_get(_:String)->Option<* mut c_char>{
            Some(CString::new(std::fs::read_to_string("tests/test.txt").unwrap()).unwrap().into_raw())
        }
        fn callback_put(_:String)->Option<* mut c_char>{
            todo!()
        }

        thread::spawn(|| {
            let _ = unsafe{run_rs(callback_get, callback_put, "127.0.0.1:9001".parse().unwrap())};
        });
        std::thread::sleep(std::time::Duration::from_millis(100));
        let file= String::from_utf8(Command::new("mktemp").output()?.stdout)?.trim().to_string();
        Command::new("curl").arg("-s").arg("-o").arg(&file).arg("tftp://127.0.0.1:9001/test").status()?;
        let out = String::from_utf8(Command::new("sha256sum").arg(&file).output()?.stdout)?;
        let sum1 = out.split(" ").collect::<Vec<&str>>()[0];
        assert_eq!(sum1,"7a32493ca5058aa7065ab15cb6f91b43193109fd87c7d8fdefb26846acf12cc2");
        Ok(())
    }
}