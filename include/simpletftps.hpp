#pragma once

#include <stddef.h>


extern "C" {
    void run(
        char* (*callback_get)(const char*, size_t*),
        char* (*callback_put)(const char*, size_t*),
        char* address
    );
}

