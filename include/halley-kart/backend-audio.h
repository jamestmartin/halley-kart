#ifndef HK_BACKEND_AUDIO_H
#define HK_BACKEND_AUDIO_H

#include "backend.h"

struct hk_backend_audio_procs {
    struct hk_backend_procs backend_procs;
    void (*do_audio)(void* state);
};

#endif /* HK_BACKEND_AUDIO_H */
