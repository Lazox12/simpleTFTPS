import simpleTFTPS
import sys
import os
import time

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <file_to_serve> [address]")
        print("Example: python serve_file.py my_image.bin 0.0.0.0:69")
        sys.exit(1)

    file_path = sys.argv[1]
    address = sys.argv[2] if len(sys.argv) > 2 else "0.0.0.0:6969"

    if not os.path.exists(file_path):
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)

    # Read the file content once to serve it
    try:
        with open(file_path, "rb") as f:
            file_data = f.read()
    except Exception as e:
        print(f"Error reading file: {e}")
        sys.exit(1)

    def cb_get(requested_file):
        print(f"[Server] Client requested: {requested_file}")
        # In this mode, we serve the same file regardless of what was requested
        # or we could check if requested_file matches basename
        return file_data

    def cb_put(requested_file):
        print(f"[Server] Put request ignored: {requested_file}")
        return None

    print(f"[Server] Starting TFTP server on {address}...")
    print(f"[Server] Serving file: {file_path} ({len(file_data)} bytes)")
    print("[Server] Press Ctrl+C to stop.")

    try:
        simpleTFTPS.run(address, cb_get, cb_put)
    except KeyboardInterrupt:
        print("\n[Server] Stopping...")

if __name__ == "__main__":
    main()
