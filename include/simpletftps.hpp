#pragma once

#include <string>

extern "C" {
    void run(
        char* (*callback_get)(const std::string*),
        char* (*callback_put)(const std::string*),
        char* address
    );
}
