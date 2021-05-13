#ifndef HK_BACKEND_DISPLAY_H
#define HK_BACKEND_DISPLAY_H

#include "backend.h"

struct hk_backend_display_procs {
    struct hk_backend_procs backend_procs;
    void* (*display_create)(void* backend);
    void (*display_destroy)(void* display);
};

#endif /* HK_BACKEND_DISPLAY_H */
