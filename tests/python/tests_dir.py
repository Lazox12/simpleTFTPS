import pytest
import simpleTFTPS
import time
import subprocess
import hashlib
import os
import threading
from pathlib import Path

# Setup the data directory
DATA_DIR = Path(__file__).parent / "data"
FILES = list(DATA_DIR.glob("*"))

# Start the server globally for these tests
SERVER_ADDRESS = "127.0.0.1:9005"
def cb_get(file_name):
    # Map the requested filename back to our data directory
    # TFTP usually just sends the basename or a path
    target_path = DATA_DIR / os.path.basename(file_name)
    if target_path.exists():
        return target_path.read_bytes()
    return None

def cb_put(file_name):
    return None



@pytest.fixture(scope="module", autouse=True)
def tftp_server():
    server_thread = threading.Thread(
        target=simpleTFTPS.run, 
        args=(SERVER_ADDRESS, cb_get, cb_put),
        daemon=True
    )
    server_thread.start()
    time.sleep(1) # Wait for server to start
    yield
    # Server thread will exit when process ends (daemon)

@pytest.mark.parametrize("file_path", FILES, ids=lambda x: x.name)
def test_tftp_file_transfer(file_path):
    output_file = Path(f"test_out_{file_path.name}")
    if output_file.exists():
        output_file.unlink()

    try:
        # Use curl to fetch the file
        # The URL contains the filename which our cb_get will use
        result = subprocess.run(
            ["curl", "-s", f"tftp://{SERVER_ADDRESS}/{file_path.name}", "-o", str(output_file)],
            capture_output=True,
            timeout=10
        )
        
        assert result.returncode == 0, f"curl failed for {file_path.name}: {result.stderr}"
        
        # Verify content
        expected_data = file_path.read_bytes()
        received_data = output_file.read_bytes()
        
        assert received_data == expected_data, f"Data mismatch for {file_path.name}"
        
        # Verify hash just to be extra sure
        expected_hash = hashlib.sha256(expected_data).hexdigest()
        actual_hash = hashlib.sha256(received_data).hexdigest()
        assert actual_hash == expected_hash
        
    finally:
        if output_file.exists():
            output_file.unlink()
