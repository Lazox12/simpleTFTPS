#include "simpletftps.hpp"
#include <iostream>
#include <thread>
#include <chrono>
#include <cstdlib>
#include <cstring>

// Simple mock file content
const char* MOCK_CONTENT = "Hello from simpleTFTPS C++ test!";

extern "C" char* tftp_get(const char* file) {
    std::cout << "[Server] Request for file: " << file << std::endl;
    // We return a pointer to the mock content.
    // The Rust side will now safely copy this and NOT try to free it.
    return (char*)MOCK_CONTENT;
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
    int ret = std::system("curl -s tftp://127.0.0.1:6969/test.txt -o test_out.txt");
    
    if (ret == 0) {
        std::cout << "[Client] Transfer successful!" << std::endl;
        
        // Verify content
        FILE* f = fopen("test_out.txt", "r");
        if (f) {
            char buffer[128];
            fgets(buffer, sizeof(buffer), f);
            fclose(f);
            if (strcmp(buffer, MOCK_CONTENT) == 0) {
                std::cout << "[Client] Content verified: " << buffer << std::endl;
                std::cout << "TEST PASSED" << std::endl;
                return 0;
            } else {
                std::cerr << "[Client] Content mismatch! Got: " << buffer << std::endl;
            }
        }
    } else {
        std::cerr << "[Client] Transfer failed with code: " << ret << std::endl;
    }

    std::cout << "TEST FAILED" << std::endl;
    return 1;
}
