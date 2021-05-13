#ifndef HK_BACKEND_GRAPHICS_H
#define HK_BACKEND_GRAPHICS_H

#include "backend.h"

struct hk_backend_graphics_procs {
    struct hk_backend_procs backend_procs;
    void (*do_graphics)(void* state);
};

#endif /* HK_BACKEND_GRAPHICS_H */
