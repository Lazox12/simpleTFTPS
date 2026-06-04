# simpleTFTPS

`simpleTFTPS` is a high-performance TFTP (Trivial File Transfer Protocol) server written in Rust, providing seamless integration for C++ and Python through native bindings. It allows developers to implement custom `GET` and `PUT` logic via simple callbacks.

- **Sources**: all sources are available at [https://gl2.aws.paperboat.cz/lazox12/pytftps](https://gl2.aws.paperboat.cz/lazox12/pytftps), or at a mirror at [https://github.com/Lazox12/simpleTFTPS](https://github.com/Lazox12/simpleTFTPS)

## Features

- **Core in Rust**: Fast, safe, and concurrent TFTP engine.
- **Unified Build**: Simple `Makefile` to build everything (Rust, C++, Python).
- **Easy Integration**: Native bindings for C++ and Python.
- **Customizable**: Simple callback system to handle file requests and uploads.
- **Multi-threaded**: Handles multiple client requests simultaneously using Rust threads.
- **Modern Python Support**: Compatible with Python 3.7+ (GIL-safe).

## Prerequisites

- **Rust**: [Installation guide](https://www.rust-lang.org/tools/install)
- **C++ Compiler**: GCC (G++) or Clang supporting C++17
- **Python**: Python 3.7 or higher (for Python bindings)
- **Make**: To use the simplified build system
- **libcurl**: Required for integration tests

## Quick Start

### Build Everything
To build the Rust library, C++ examples, and Python extension:
```bash
make
```

### Run All Tests
To run Rust unit tests, C++ integration tests, and Python integration tests:
```bash
make test
```

### Clean Up
To remove all build artifacts:
```bash
make clean
```

## Usage

### C++
Include the header and link against the Rust static library. Use the `run()` function with your callbacks.
```cpp
#include "simpletftps.hpp"

extern "C" char* my_get_callback(const char* file) {
    return strdup("Data content");
}

int main() {
    run(my_get_callback, nullptr, "127.0.0.1:6969");
    return 0;
}
```

### Python
Import the module and run the server in a thread (as `run` is blocking).
```python
import simpleTFTPS
import threading

def cb_get(file):
    return "File content"

threading.Thread(target=simpleTFTPS.run, args=("127.0.0.1:9001", cb_get, None), daemon=True).start()
```

## Project Structure

- `simpleTFTPS/`: Rust implementation of the TFTP engine.
- `include/`: C++ header and Python binding implementation.
- `tests/`: Integration tests for C++, Python, and Rust.
- `Makefile`: Unified build system.

## License

This project is licensed under the MIT License.
