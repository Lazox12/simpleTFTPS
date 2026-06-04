#include "simpletftps.hpp"
#include <fstream>
#include <unistd.h>
#include <array>
#include <assert.h>
#include <iostream>
char address[] = "127.0.0.1:6969";

std::string run_command(const char* cmd) {

    std::array<char, 128> buffer;
    std::string result;

    // 1. Open the pipe using popen
    // Note: Use _popen on Windows (MSVC)
    FILE* pipe = popen(cmd, "r");
    if (!pipe) {
        throw std::runtime_error("popen() failed!");
    }

    // 2. Read from the pipe until it's empty
    while (fgets(buffer.data(), buffer.size(), pipe) != nullptr) {
        result += buffer.data();
    }

    // 3. You MUST use pclose(), never fclose()
    int return_code = pclose(pipe);

    if (return_code != 0) {
        std::cerr << "Command exited with code: " << return_code << '\n';
    }

    return result;
}

extern "C" char* tftp_get(const std::string* file) {
    std::ifstream fs("../tests/c/test.txt");
    if (fs.is_open()) {
        fs.seekg(0, std::ios::end);
        std::streamsize size = fs.tellg();
        if (size <= 0) {
            return nullptr;
        }
        fs.seekg(0, std::ios::beg);

        char* buffer = new char[size + 1];

        if (fs.read(buffer, size)) {
            buffer[size] = '\0'; // Safely add the null terminator
        }
        return buffer;
    }
    return nullptr;
}

extern "C" char* tftp_put(const std::string* file) {
    return nullptr;
}

int main() {
    if (fork()==0) {
        const std::string file = run_command("mktemp");
        std::cout<<"file: "<<file<<std::endl;
        run_command(("curl -s tftp://127.0.0.1:6969/test -o"+file).c_str());
        const std::string sum = run_command(("sha256sum "+file).c_str()).substr(0, 64);
        assert(sum=="7a32493ca5058aa7065ab15cb6f91b43193109fd87c7d8fdefb26846acf12cc2");
    }
    run(tftp_get, tftp_put, address);
    return 0;
}
