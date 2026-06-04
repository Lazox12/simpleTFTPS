import pytest
import simpleTFTPS
import time
import subprocess
import hashlib
import os
import threading

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

def test_blksize_option():
    # Create a reasonably sized file to test multiple blocks
    data_size = 4096
    binary_data = os.urandom(data_size)
    
    def cb_get(file):
        return binary_data
            
    def cb_put(file):
        return None
    
    # Use a fresh port
    server_thread = threading.Thread(
        target=simpleTFTPS.run, 
        args=("127.0.0.1:9003", cb_get, cb_put),
        daemon=True
    )
    server_thread.start()
    
    time.sleep(1)

    output_file = "test_blksize_out.bin"
    if os.path.exists(output_file):
        os.remove(output_file)

    try:
        # Pass --tftp-blksize 1024 to curl
        result = subprocess.run(
            ["curl", "-s", "--tftp-blksize", "1024", "tftp://127.0.0.1:9003/large.bin", "-o", output_file],
            capture_output=True,
            timeout=5
        )
        
        assert result.returncode == 0, f"curl failed: {result.stderr}"
        
        with open(output_file, "rb") as f:
            received_data = f.read()
            
        assert received_data == binary_data, f"Received data size {len(received_data)} does not match expected {len(binary_data)}"
    finally:
        if os.path.exists(output_file):
            os.remove(output_file)
