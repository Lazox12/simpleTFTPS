use std::env;
use std::fs::File;
use std::io::Read;
use simpleTFTPS::run_rs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_to_serve> [address]", args[0]);
        eprintln!("Example: {} my_image.bin 0.0.0.0:69", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    let address = if args.len() > 2 {
        args[2].clone()
    } else {
        "0.0.0.0:6969".to_string()
    };

    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: File '{}' not found or could not be opened: {}", file_path, e);
            std::process::exit(1);
        }
    };

    let mut file_data = Vec::new();
    if let Err(e) = file.read_to_end(&mut file_data) {
        eprintln!("Error reading file: {}", e);
        std::process::exit(1);
    }

    let data_len = file_data.len();
    
    // Using an Arc would be nice, but since the closures need to be 'static + Copy,
    // and simpleTFTPS::run_rs requires Copy for the closures, we can either:
    // a) clone the data inside the closure (if we store it in a static or leak it)
    // b) Box::leak the Vec so we have a &'static [u8] and just return it by cloning into a Vec when requested.
    
    // Let's leak the data to get a static slice, since the server runs forever anyway
    let leaked_data: &'static [u8] = Box::leak(file_data.into_boxed_slice());

    let cb_get = move |requested_file: String| -> Option<Vec<u8>> {
        println!("[Server] Client requested: {}", requested_file);
        // We serve the same file regardless of the request in this standalone test mode
        Some(leaked_data.to_vec())
    };

    let cb_put = |requested_file: String| -> Option<Vec<u8>> {
        println!("[Server] Put request ignored: {}", requested_file);
        None
    };

    println!("[Server] Starting TFTP server on {}...", address);
    println!("[Server] Serving file: {} ({} bytes)", file_path, data_len);
    println!("[Server] Press Ctrl+C to stop.");

    unsafe {
        if let Err(e) = run_rs(cb_get, cb_put, address) {
            eprintln!("\n[Server] Error running server: {:?}", e);
        }
    }
}
