#[cfg(test)]
mod tests {
    use simpleTFTPS::run_rs;
    use simpleTFTPS::error::Error;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_read() -> Result<(), Error> {
        let callback_get = |_: String| -> Option<Vec<u8>> {
            std::fs::read("tests/test.txt").ok()
        };

        let callback_put = |_: String| -> Option<Vec<u8>> {
            None
        };

        thread::spawn(move || {
            unsafe {
                let _ = run_rs(callback_get, callback_put, "127.0.0.1:9002".to_string());
            }
        });

        thread::sleep(Duration::from_millis(500));

        let output = Command::new("mktemp").output().map_err(|e| Error::Str(e.to_string()))?;
        let file_path = String::from_utf8(output.stdout).map_err(|e| Error::Str(e.to_string()))?.trim().to_string();

        let status = Command::new("curl")
            .arg("-s")
            .arg("-o")
            .arg(&file_path)
            .arg("tftp://127.0.0.1:9002/test")
            .status()
            .map_err(|e| Error::Str(e.to_string()))?;

        assert!(status.success(), "curl command failed");

        let sha_output = Command::new("sha256sum")
            .arg(&file_path)
            .output()
            .map_err(|e| Error::Str(e.to_string()))?;
        
        let out_str = String::from_utf8(sha_output.stdout).map_err(|e| Error::Str(e.to_string()))?;
        let sum1 = out_str.split_whitespace().next().unwrap_or("");

        assert_eq!(sum1, "7a32493ca5058aa7065ab15cb6f91b43193109fd87c7d8fdefb26846acf12cc2");

        // Cleanup
        let _ = std::fs::remove_file(&file_path);

        Ok(())
    }
}
