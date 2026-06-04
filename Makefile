# simpleTFTPS Makefile

# Configurable variables
CXX = g++
CARGO = cargo
PYTHON = python3
PYTHON_CONFIG = python3-config

# Paths
RUST_DIR = simpleTFTPS
RUST_LIB = $(RUST_DIR)/target/release/libsimpleTFTPS.a
INCLUDE_DIR = include
TEST_C_DIR = tests/c
TEST_PY_DIR = tests/python
BUILD_DIR = build

# Compilation flags
CXXFLAGS = -O3 -Wall -Wextra -std=c++17 -I$(INCLUDE_DIR)
PYTHON_CFLAGS = $(shell $(PYTHON_CONFIG) --cflags)
PYTHON_LDFLAGS = $(shell $(PYTHON_CONFIG) --ldflags)
PYTHON_SUFFIX = $(shell $(PYTHON_CONFIG) --extension-suffix)

# Targets
PYTHON_SO = $(BUILD_DIR)/simpleTFTPS$(PYTHON_SUFFIX)
CPP_TEST_BIN = $(BUILD_DIR)/simpleTFTPS-c

.PHONY: all clean rust cpp python test help

all: $(BUILD_DIR) rust cpp python

help:
	@echo "Available targets:"
	@echo "  all      - Build Rust library, C++ test, and Python extension (default)"
	@echo "  rust     - Build the Rust core library"
	@echo "  cpp      - Build the C++ test/example executable"
	@echo "  python   - Build the Python extension module"
	@echo "  test     - Run both C++ and Python tests"
	@echo "  clean    - Remove build artifacts"

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

rust:
	cd $(RUST_DIR) && $(CARGO) build --release

cpp: rust $(BUILD_DIR)
	$(CXX) $(CXXFLAGS) $(TEST_C_DIR)/main.cpp $(RUST_LIB) -lpthread -ldl -o $(CPP_TEST_BIN)

python: rust $(BUILD_DIR)
	$(CXX) $(CXXFLAGS) -shared -fPIC $(INCLUDE_DIR)/simpletftps.cpp $(PYTHON_CFLAGS) $(RUST_LIB) -lpthread -ldl -o $(PYTHON_SO)

rust_test:
	cd $(RUST_DIR) && $(CARGO) test

test: all rust_test
	@echo "Running C++ tests..."
	./$(CPP_TEST_BIN)
	@echo "Running Python tests..."
	PYTHONPATH=$(BUILD_DIR) .venv/bin/python -m pytest $(TEST_PY_DIR)/tests.py

clean:
	rm -rf $(BUILD_DIR)
	rm -rf cmake-build-debug
	cd $(RUST_DIR) && $(CARGO) clean
	rm -f test_out.txt test_out_py.txt
	rm -f simpleTFTPS.cpython-314-x86_64-linux-gnu.so simpleTFTPS-c
