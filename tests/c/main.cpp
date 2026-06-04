#include <assert.h>

#include "simpletftps.hpp"
#include <iostream>
#include <thread>
#include <chrono>
#include <cstdlib>
#include <cstring>
#include <fstream>

// Simple mock file content
const char* MOCK_CONTENT = "Hello from simpleTFTPS C++ test!";

extern "C" char* tftp_get(const char* file) {
    std::cout << "[Server] Request for file: " << file << std::endl;
    std::ifstream file_stream("tests/c/test.txt");
    if (file_stream.is_open()) {
        std::string content((std::istreambuf_iterator<char>(file_stream)), std::istreambuf_iterator<char>());
        return strdup(content.c_str());
    }
    return nullptr;
}

extern "C" char* tftp_put(const char* file) {
    std::cout << "[Server] Putting file: " << file << std::endl;
    return nullptr;
}

void run_server() {
    char address[] = "127.0.0.1:6969";
    std::cout << "[Server] Starting on " << address << "..." << std::endl;
    run(tftp_get, tftp_put, address);
}

int main() {
    // Start server in background
    std::thread server_thread(run_server);
    server_thread.detach();

    // Wait for server to bind
    std::this_thread::sleep_for(std::chrono::milliseconds(500));

    std::cout << "[Client] Attempting to fetch file via curl..." << std::endl;
    
    // Use curl to fetch the file. We expect it to succeed now.
    // tftp://host:port/path
    std::system("curl -s tftp://127.0.0.1:6969/test.txt -o test_out.txt");

    FILE* out = popen("sha256sum test_out.txt","r");
    const auto str = static_cast<char *>(malloc(65));
    fgets(str, 65, out);
    std::cout << "[Client] Hash: " << str << std::endl;
    assert(std::string(str)=="7a32493ca5058aa7065ab15cb6f91b43193109fd87c7d8fdefb26846acf12cc2");
    free(str);
    return 0;
}
