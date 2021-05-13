#include <dlfcn.h>
#include <elf.h>
#include <errno.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <halley-kart/backend.h>
#include <halley-kart/plugin-manager.h>
#include <halley-kart/plugin-manager-client.h>

#ifndef align_up
#define align_down(num, align) ((num) & ~((align) - 1))
#define align_up(num, align) (align_down((num) + (align) - 1, align))
#endif

const struct hk_plugin_dll NULL_PLUGIN_DLL = { NULL, NULL };

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wpointer-arith"
static void* mmap_realigned(size_t size, int fd, size_t off) {
    size_t extra = off - align_down(off, 4096);
    void* map = mmap(
        NULL,
        size + extra,
        PROT_READ,
        MAP_PRIVATE,
        fd,
        align_down(off, 4096)
    );
    return map + extra;
}

static void munmap_realigned(void* ptr, size_t size) {
    munmap((void*) align_down((size_t) ptr, 4096), align_up(size, 4096));
}

struct hk_plugin_dll hk_plugin_load_dll(const char* path) {
    //
    // Read ELF header in advance to:
    //   1. Find out the get_procs symbol for this plugin
    //   2. Find other plugin metadata
    //
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        fprintf(
            stderr,
            "ERROR: failed to open plugin file %s: %s\n",
            path,
            strerror(errno)
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }

    size_t length = lseek(fd, 0, SEEK_END);
    if (length == (size_t) -1) {
        fprintf(
            stderr,
            "ERROR: failed to get size of plugin file %s: %s\n",
            path,
            strerror(errno)
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    if (length < sizeof(Elf64_Ehdr)) {
        fprintf(stderr, "ERROR: plugin ELF is too small %s\n", path);
        close(fd);
        return NULL_PLUGIN_DLL;
    }

    // Parse ELF header
    Elf64_Ehdr* ehdr = mmap_realigned(sizeof(Elf64_Ehdr), fd, 0);
    if (ehdr == (void*) -1) {
        fprintf(
            stderr,
            "ERROR: failed to map plugin ELF header %s: %s\n",
            path,
            strerror(errno)
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    size_t magic_length = 7;
    char magic[] = { 0x7F, 'E', 'L', 'F', ELFCLASS64, ELFDATA2LSB, EV_CURRENT };
    if (memcmp(ehdr, magic, magic_length) != 0) {
        fprintf(stderr, "ERROR: plugin ELF lacks magic %s\n", path);
        munmap_realigned(ehdr, sizeof(Elf64_Ehdr));
        close(fd);
        return NULL_PLUGIN_DLL;
    }

    Elf64_Off shoff = ehdr->e_shoff;
    uint16_t shnum = ehdr->e_shnum;
    uint16_t shentsize = ehdr->e_shentsize;
    uint16_t shstrndx = ehdr->e_shstrndx;
    munmap_realigned(ehdr, sizeof(Elf64_Ehdr));

    // Parse section table
    if (shnum == 0) {
        fprintf(stderr, "ERROR: plugin ELF contains no sections %s\n", path);
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    if (shstrndx > shnum) {
        fprintf(stderr, "ERROR: plugin ELF shstrndx exceeds shnum %s\n", path);
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    if (shentsize < sizeof(Elf64_Shdr)) {
        fprintf(
            stderr,
            "ERROR: plugin ELF shentsize smaller than sizeof(Elf64_Shdr) %s\n",
            path
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    if (shoff + shnum * shentsize > length) {
        fprintf(
            stderr,
            "ERROR: plugin ELF section headers exceed file size %s\n",
            path
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    size_t shdrs_size = shentsize * shnum;
    Elf64_Shdr* shdrs = mmap_realigned(shdrs_size, fd, shoff);
    if (shdrs == (void*) -1) {
        fprintf(
            stderr,
            "ERROR: failed to map plugin ELF section table %s\n",
            path
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    Elf64_Shdr* shdrshstrs = (void*) shdrs + shstrndx * shentsize;
    Elf64_Off shstrs_offset = shdrshstrs->sh_offset;
    uint64_t shstrs_size = shdrshstrs->sh_size;
    if (shstrs_offset + shstrs_size > length) {
        fprintf(
            stderr,
            "ERROR: plugin ELF section header string table exeeds file size %s\n",
            path
        );
        munmap_realigned(shdrs, shdrs_size);
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    char* shstrs = mmap_realigned(shstrs_size, fd, shstrs_offset);
    if (shstrs == (void*) -1) {
        munmap_realigned(shdrs, shdrs_size);
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    Elf64_Shdr* shdrhk_plugin = NULL;
    for (uint16_t i = 0; i < shnum; i++) {
        Elf64_Shdr* shdr = (void*) shdrs + i * shentsize;
        switch (shdr->sh_type) {
        case SHT_NOTE:
            if (shdr->sh_name > shstrs_size) {
                fprintf(
                    stderr,
                    "ERROR: plugin ELF section name exceeds shstrs size %s\n",
                    path
                );
                munmap_realigned(shstrs, shstrs_size);
                munmap_realigned(shdrs, shdrs_size);
                close(fd);
                return NULL_PLUGIN_DLL;
            }
            char* name = &shstrs[shdr->sh_name];
            size_t bound = shstrs_size - shdr->sh_name;
            if (strncmp("HK-plugin", name, bound) != 0) {
                continue;
            }
            if (shdrhk_plugin != NULL) {
                fprintf(
                    stderr,
                    "ERROR: plugin ELF contains multiple HK-plugin sections %s\n",
                    path
                );
                munmap_realigned(shstrs, shstrs_size);
                munmap_realigned(shdrs, shdrs_size);
                close(fd);
                return NULL_PLUGIN_DLL;
            }
            if (shdr->sh_offset + shdr->sh_size > length) {
                fprintf(
                    stderr,
                    "ERROR: plugin ELF HK-plugin section exceeds file size %s\n",
                    path
                );
                munmap_realigned(shstrs, shstrs_size);
                munmap_realigned(shdrs, shdrs_size);
                close(fd);
                return NULL_PLUGIN_DLL;
            }
            shdrhk_plugin = shdr;

            goto sections_found;
        }
    }
    munmap_realigned(shstrs, shstrs_size);
    if (shdrhk_plugin == NULL) {
        fprintf(stderr, "ERROR: plugin ELF missing HK-plugin section %s\n", path);
        munmap_realigned(shdrs, shdrs_size);
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    sections_found:
    munmap_realigned(shstrs, shstrs_size);
    Elf64_Off shhk_plugin_offset = shdrhk_plugin->sh_offset;
    uint64_t shhk_plugin_size = shdrhk_plugin->sh_size;
    munmap_realigned(shdrs, shdrs_size);

    // Parse plugin notes section
    Elf64_Nhdr* hk_plugin_notes =
        mmap_realigned(shhk_plugin_size, fd, shhk_plugin_offset);
    if (hk_plugin_notes == (void*) -1) {
        fprintf(
            stderr,
            "ERROR: failed to map plugin HK-plugin section %s\n",
            path
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }
    char* get_procs_symbol = NULL;
    _Bool optional_dynamic = false;
    size_t off = 0;
    while (off + sizeof(Elf64_Nhdr) < shhk_plugin_size) {
        Elf64_Nhdr* nhdr = (void*) hk_plugin_notes + off;
        off += sizeof(Elf64_Nhdr);
        char* name = (char*) ((void*) hk_plugin_notes + off);
        off += align_up(nhdr->n_namesz, 4);
        char* desc = (char*) ((void*) hk_plugin_notes + off);
        off += align_up(nhdr->n_descsz, 4);
        if (off > shhk_plugin_size) {
            fprintf(
                stderr,
                "ERROR: plugin HK-plugin descriptor exceeds section size %s\n",
                path
            );
            munmap_realigned(hk_plugin_notes, shhk_plugin_size);
            close(fd);
            return NULL_PLUGIN_DLL;
        }
        if (
            nhdr->n_namesz != strlen(HK_PLUGIN_NOTE_NAME) + 1
            || memcmp(name, HK_PLUGIN_NOTE_NAME, nhdr->n_namesz) != 0
        ) {
            continue;
        }
        switch (nhdr->n_type) {
        case HK_PLUGIN_NOTE_TYPE_GET_PROCS:
            get_procs_symbol = desc;
            if (nhdr->n_descsz < 2) {
                fprintf(
                    stderr,
                    "ERROR: plugin HK-plugin get_procs note is empty %s\n",
                    path
                );
                munmap_realigned(hk_plugin_notes, shhk_plugin_size);
                close(fd);
                return NULL_PLUGIN_DLL;
            }
            if (desc[nhdr->n_descsz - 1] != '\0') {
                fprintf(
                    stderr,
                    "ERROR: plugin HK-plugin get_procs note"
                    "is not null-terminted %s\n",
                    path
                );
                munmap_realigned(hk_plugin_notes, shhk_plugin_size);
                close(fd);
                return NULL_PLUGIN_DLL;
            }
            break;
        case HK_PLUGIN_NOTE_TYPE_OPTIONAL_DYNAMIC:
            optional_dynamic = true;
            break;
        }
    }
    if (get_procs_symbol == NULL) {
        fprintf(
            stderr,
            "ERROR: plugin HK-plugin section does not contain get_procs note %s\n",
            path
        );
        close(fd);
        return NULL_PLUGIN_DLL;
    }

    struct hk_plugin_dll plugin_dll;

    // Open the shared object
    plugin_dll.dll_handle = dlopen(path, RTLD_NOW | RTLD_LOCAL);
    close(fd);
    if (plugin_dll.dll_handle == NULL) {
        char* severity = optional_dynamic ? "WARN" : "ERROR";
        fprintf(
            stderr,
            "%s: failed to load plugin %s: %s\n",
            severity,
            path,
            dlerror()
        );
        munmap_realigned(hk_plugin_notes, shhk_plugin_size);
        return NULL_PLUGIN_DLL;
    }

    const struct hk_plugin_procs* (*get_procs)(void);
#pragma GCC diagnostic push
// ISO C does not allow casting `void*` to a function pointer,
// which is inherently incompatible with `dlsym`.
#pragma GCC diagnostic ignored "-Wpedantic"
    get_procs = dlsym(plugin_dll.dll_handle, get_procs_symbol);
#pragma GCC diagnostic pop
    if (get_procs == NULL) {
        fprintf(
            stderr,
            "ERROR: could not load plugin get_procs symbol %s %s: %s\n",
            path,
            get_procs_symbol,
            dlerror()
        );
        munmap_realigned(hk_plugin_notes, shhk_plugin_size);
        return NULL_PLUGIN_DLL;
    }
    munmap_realigned(hk_plugin_notes, shhk_plugin_size);

    plugin_dll.procs = get_procs();

    return plugin_dll;
}

static void** array_extend(
    void** arr,
    size_t* size,
    size_t len
) {
    if (*arr == NULL) {
        *arr = malloc(sizeof(void*) * 32);
        *size = 32;
        return *arr;
    }
    if (len >= *size) {
        *size *= 32;
        *arr = realloc(*arr, *size * sizeof(void*));
    }
    return *arr + len * sizeof(void*);
}

static size_t array_indexof(
    void** arr,
    size_t len,
    const void* value
) {
    for (size_t i = 0; i < len; i++) {
        void* x = arr[i];
        if (x == value) {
            return i;
        }
    }
    return (size_t) -1;
}

static void array_remove(
    void** arr,
    size_t* len,
    size_t index
) {
    memmove(
        &arr[index],
        &arr[index + 1],
        (*len - index) * sizeof(void*)
    );
    (*len)--;
}

#pragma GCC diagnostic pop

void hk_plugin_unload_dll(struct hk_plugin_dll dll) {
    dlclose(dll.dll_handle);
}

struct hk_backend_handle {
    struct hk_plugin_handle* plugin;
    const struct hk_backend_procs* procs;
};

struct hk_plugin_handle {
    const struct hk_plugin_procs* procs;
};

struct hk_plugin_manager {
    struct hk_plugin_handle** plugins;
    size_t plugins_size;
    size_t plugins_len;
    struct hk_backend_handle** backends;
    size_t backends_size;
    size_t backends_len;
};

const struct hk_backend_procs* hk_backend_get_procs(
    const struct hk_backend_handle* handle
) {
    return handle->procs;
}

struct hk_plugin_handle* hk_backend_get_plugin(
    const struct hk_backend_handle* handle
) {
    return handle->plugin;
}

struct hk_plugin_handle* hk_plugin_register(
    struct hk_plugin_manager* mngr,
    const struct hk_plugin_procs* procs
) {
    struct hk_plugin_handle** handle_ptr =
        (struct hk_plugin_handle**) array_extend(
            (void**) &mngr->plugins,
            &mngr->plugins_size,
            mngr->plugins_len
        );
    mngr->plugins_len++;
    struct hk_plugin_handle* handle = malloc(sizeof(struct hk_plugin_handle));
    *handle_ptr = handle;
    handle->procs = procs;
    return handle;
}

void hk_plugin_unregister(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin
) {
    size_t index = array_indexof(
        (void**) mngr->plugins,
        mngr->plugins_len,
        plugin
    );
    if (index == (size_t) -1) {
        fprintf(stderr, "BUG: tried to remove a nonexistent plugin\n");
        return;
    }
    array_remove((void**) mngr->plugins, &mngr->plugins_len, index);
    free(plugin);
}

struct hk_backend_handle* hk_plugin_register_backend(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin,
    const struct hk_backend_procs* procs
) {
    struct hk_backend_handle** handle_ptr =
        (struct hk_backend_handle**) array_extend(
            (void**) &mngr->backends,
            &mngr->backends_size,
            mngr->backends_len
        );
    mngr->backends_len++;
    struct hk_backend_handle* handle = malloc(sizeof(struct hk_backend_handle));
    *handle_ptr = handle;
    handle->plugin = plugin;
    handle->procs = procs;
    return handle;
}

extern void hk_plugin_unregister_backend(
    struct hk_plugin_manager* mngr,
    struct hk_backend_handle* backend
) {
    size_t index = array_indexof(
        (void**) mngr->backends,
        mngr->backends_len,
        backend
    );
    if (index == (size_t) -1) {
        fprintf(stderr, "BUG: tried to remove a nonexistent plugin backend\n");
        return;
    }
    array_remove((void**) mngr->backends, &mngr->backends_len, index);
    free(backend);
}

size_t hk_plugin_enumerate_plugins(
    struct hk_plugin_manager* mngr,
    size_t len,
    struct hk_plugin_handle** plugins
) {
    if (mngr->plugins == NULL) {
        return 0;
    }
    if (plugins == NULL) {
        return mngr->plugins_len;
    }
    size_t actual_len = len > mngr->plugins_len ? mngr->plugins_len : len;
    memcpy(plugins, mngr->plugins, actual_len);
    return actual_len;
}

size_t hk_plugin_enumerate_backends(
    struct hk_plugin_manager* mngr,
    enum hk_backend_type type,
    size_t len,
    struct hk_backend_handle** backends
) {
    if (mngr->backends == NULL) {
        return 0;
    }
    if (backends == NULL) {
        size_t count = 0;
        for (size_t i = 0; i < mngr->backends_len; i++) {
            struct hk_backend_handle* backend = mngr->backends[i];
            if (backend->procs->type == type) {
                count++;
            }
        }
        return count;
    }
    size_t count = 0;
    for (size_t i = 0; i < mngr->backends_len && count < len; i++) {
        struct hk_backend_handle* backend = mngr->backends[i];
        if (backend->procs->type == type) {
            backends[count++] = backend;
        }
    }
    return count;
}

struct hk_plugin_manager* hk_plugin_manager_create(void) {
    struct hk_plugin_manager* mngr = malloc(sizeof(struct hk_plugin_manager));
    mngr->plugins = NULL;
    mngr->backends = NULL;
    return mngr;
}

void hk_plugin_manager_destroy(struct hk_plugin_manager* mngr) {
    for (size_t i = 0; i < mngr->backends_len; i++) {
        free(mngr->backends[i]);
    }
    free(mngr->backends);
    for (size_t i = 0; i < mngr->plugins_len; i++) {
        free(mngr->plugins[i]);
    }
    free(mngr->plugins);
    free(mngr);
}
