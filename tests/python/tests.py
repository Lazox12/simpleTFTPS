import pytest
import simpleTFTPS
import time
import subprocess
import hashlib
import os
import threading

def test_run():
    # Use absolute path for test.txt to be sure
    test_file_path = os.path.join(os.path.dirname(__file__), "ipxe.efi")
    
    def cb_get(file):
        try:
            # TFTP can be binary, but here we expect text
            with open(test_file_path, "r") as f:
                return f.read()
        except Exception as e:
            print(f"Error in cb_get: {e}")
            return None
            
    def cb_put(file):
        return None
    
    # Run the server in a separate thread because it's now blocking
    server_thread = threading.Thread(
        target=simpleTFTPS.run, 
        args=("127.0.0.1:9001", cb_get, cb_put),
        daemon=True
    )
    server_thread.start()
    
    time.sleep(1) # Give it time to start

    output_file = "test_out_py.txt"
    if os.path.exists(output_file):
        os.remove(output_file)

    try:
        # Use curl like in the C test
        result = subprocess.run(
            ["curl", "-s", "tftp://127.0.0.1:9001/test.txt", "-o", output_file],
            capture_output=True,
            text=True,
            timeout=5
        )
        
        assert result.returncode == 0, f"curl failed: {result.stderr}"
        
        with open(output_file, "rb") as f:
            body = f.read()
            
        expected_hash = "d6941b99ef404ccc7d5f129b0889a84fddea3dabc51a34d3e59fd9f9154e37bc"
        actual_hash = hashlib.sha256(body).hexdigest()
        assert actual_hash == expected_hash
    finally:
        # We can't easily stop the server thread as it's blocked in Rust
        # but since it's a daemon thread, it will exit when the test process exits.
        if os.path.exists(output_file):
            os.remove(output_file)

def test_binary_null_bytes():
    binary_data = b"Hello\x00World\x00Binary\xffData"
    
    def cb_get(file):
        # file should still be a normal string
        assert isinstance(file, str)
        return binary_data
            
    def cb_put(file):
        return None
    
    server_thread = threading.Thread(
        target=simpleTFTPS.run, 
        args=("127.0.0.1:9002", cb_get, cb_put),
        daemon=True
    )
    server_thread.start()
    
    time.sleep(1)

    output_file = "test_binary_out.bin"
    if os.path.exists(output_file):
        os.remove(output_file)

    try:
        result = subprocess.run(
            ["curl", "-s", "tftp://127.0.0.1:9002/binary.bin", "-o", output_file],
            capture_output=True,
            timeout=5
        )
        
        assert result.returncode == 0, f"curl failed: {result.stderr}"
        
        with open(output_file, "rb") as f:
            received_data = f.read()
            
        assert received_data == binary_data, f"Received data {received_data} does not match expected {binary_data}"
    finally:
        if os.path.exists(output_file):
            os.remove(output_file)
