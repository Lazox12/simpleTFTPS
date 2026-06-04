#pragma once

#include <string>

extern "C" {
    void run(
        char* (*callback_get)(const char*),
        char* (*callback_put)(const char*),
        char* address
    );
}
