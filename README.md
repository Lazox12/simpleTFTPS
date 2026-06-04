# simpleTFTPS

`simpleTFTPS` is a high-performance TFTP (Trivial File Transfer Protocol) server written in Rust, providing seamless integration for C++ and Python through native bindings. It allows developers to implement custom `GET` and `PUT` logic via simple callbacks.

## Features

- **Core in Rust**: Fast, safe, and concurrent TFTP engine.
- **Easy Integration**: Native bindings for C++ and Python.
- **Customizable**: Simple callback system to handle file requests and uploads.
- **Multi-threaded**: Handles multiple client requests simultaneously using Rust threads.
- **Modern Python Support**: Compatible with Python 3.14+ (GIL-safe).

## Prerequisites

- **Rust**: [Installation guide](https://www.rust-lang.org/tools/install)
- **CMake**: Version 3.20 or higher
- **C++ Compiler**: GCC (G++) or Clang supporting C++17
- **Python**: 3.14 or higher (for Python bindings)
- **libcurl**: Used for the provided test suite

## Installation & Build

### 1. Build the Rust Core
First, compile the Rust static library:
```bash
cd simpleTFTPS
cargo build --release
```
This generates `libsimpleTFTPS.a` in `simpleTFTPS/target/release/`.

### 2. Build C++ Bindings & Examples
Using CMake:
```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make
```
This produces the `simpleTFTPS-c` executable, which serves as a test/example.

### 3. Build Python Bindings
You can compile the Python extension manually or using `setup.py`:
```bash
# Manual compilation (ensure paths match your system)
g++ -O3 -shared -fPIC include/simpletftps.cpp -Iinclude -I/usr/include/python3.14 \
    simpleTFTPS/target/release/libsimpleTFTPS.a -lpthread -ldl \
    -o simpleTFTPS.cpython-314-x86_64-linux-gnu.so
```

## Usage

### C++ Example
```cpp
#include "simpletftps.hpp"
#include <iostream>
#include <cstring>

extern "C" char* my_get_callback(const char* file) {
    std::cout << "Client requested: " << file << std::endl;
    return strdup("Hello from C++ callback!");
}

extern "C" char* my_put_callback(const char* file) {
    std::cout << "Client uploading: " << file << std::endl;
    return nullptr; // No-op
}

int main() {
    char address[] = "127.0.0.1:6969";
    run(my_get_callback, my_put_callback, address);
    return 0;
}
```

### Python Example
```python
import simpleTFTPS
import threading
import time

def cb_get(file):
    print(f"GET request for: {file}")
    return "Content for the client"

def cb_put(file):
    print(f"PUT request for: {file}")
    return None

# The run method is blocking, use a thread for background execution
server_thread = threading.Thread(
    target=simpleTFTPS.run, 
    args=("127.0.0.1:9001", cb_get, cb_put),
    daemon=True
)
server_thread.start()

# Keep the main thread alive or perform other tasks
while True:
    time.sleep(1)
```

## Running Tests

### C++ Tests
```bash
cd cmake-build-debug
./simpleTFTPS-c
```

### Python Tests
```bash
pytest tests/python/tests.py
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details (if applicable).
