cmake_minimum_required(VERSION 3.20)
project(simpleTFTPS-test)

set(CMAKE_CXX_STANDARD 17)

# Rust library configuration
set(RUST_DIR "${CMAKE_CURRENT_SOURCE_DIR}/simpleTFTPS")
set(RUST_LIB_NAME "simpleTFTPS")

if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(CARGO_FLAGS "--release")
    set(CARGO_OUTPUT_DIR "release")
else()
    set(CARGO_FLAGS "")
    set(CARGO_OUTPUT_DIR "debug")
endif()

set(RUST_STATIC_LIB "${RUST_DIR}/target/${CARGO_OUTPUT_DIR}/lib${RUST_LIB_NAME}.a")

# Custom target that always runs cargo build
add_custom_target(rust_target ALL
    COMMAND cargo build ${CARGO_FLAGS}
    WORKING_DIRECTORY "${RUST_DIR}"
    COMMENT "Building Rust static library"
    VERBATIM
)

# Add the executable
add_executable(simpleTFTPS-test tests/c/main.cpp include/simpletftps.hpp)

# Ensure rust_target is built before the executable
add_dependencies(simpleTFTPS-test rust_target)

# Link the Rust library and its dependencies
target_link_libraries(simpleTFTPS-test PRIVATE "${RUST_STATIC_LIB}" pthread dl)

# Include directories
target_include_directories(simpleTFTPS-test PRIVATE include)
