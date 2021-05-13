#define _POSIX_C_SOURCE 200112L

#include <elf.h>
#include <errno.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <halley-kart/plugin.h>

#ifndef align_up
#define align_up(num, align) (((num) + ((align)-1)) & ~((align)-1))
#endif

static const char* const usage =
    "usage: hk-make-plugin-elf-notes [--optional-dynamic] "
    "<output file> <get_procs symbol>\n"
    ;

int main(int argc, char** argv) {\
    if (argc < 3) {
        fprintf(stderr, "not enough arguments\n%s", usage);
        return 1;
    }

    _Bool optional_dynamic = false;
    int i;
    for (i = 1; i < argc - 2; i++) {
        if (strcmp("--optional-dynamic", argv[i]) == 0) {
            optional_dynamic = true;
        } else {
            fprintf(stderr, "unknown option: %s\n%s", argv[i], usage);
            return 1;
        }
    }

    char* output_file = argv[i];
    char* get_procs_symbol = argv[i + 1];

    // only write is needed, but mmap requires O_RDWR
    int fd = open(output_file, O_RDWR | O_CREAT | O_TRUNC);
    if (fd < 0) {
        fprintf(
            stderr,
            "failed to open output file %s: %s\n%s",
            output_file,
            strerror(errno),
            usage
        );
        return 1;
    }

    size_t note_name_size = strlen(HK_PLUGIN_NOTE_NAME) + 1;
    size_t get_procs_symbol_size = strlen(get_procs_symbol) + 1;

    char* shstrtab_data = "\0.shstrtab\0HK-plugin\0";
    uint32_t hk_plugin_name_index = 11;
    uint8_t shstrtab_data_size = 21;

    Elf64_Off shtab_off = sizeof(Elf64_Ehdr);
    Elf64_Off shstrtab_hdr_off = shtab_off + sizeof(Elf64_Shdr);
    Elf64_Off hk_plugin_hdr_off = shstrtab_hdr_off + sizeof(Elf64_Shdr);
    Elf64_Off shstrtab_data_off = hk_plugin_hdr_off + sizeof(Elf64_Shdr);
    Elf64_Off hk_plugin_data_off = shstrtab_data_off + shstrtab_data_size;

    size_t hk_plugin_get_procs_size =
        sizeof(Elf64_Nhdr)
        + align_up(note_name_size, 4)
        + align_up(get_procs_symbol_size, 4)
        ;
    size_t hk_plugin_optional_dynamic_size =
        optional_dynamic ? sizeof(Elf64_Nhdr) + note_name_size : 0;
    Elf64_Off hk_plugin_optional_dynamic_off =
        hk_plugin_data_off + hk_plugin_get_procs_size;

    off_t file_size =
        hk_plugin_optional_dynamic_off
        + align_up(hk_plugin_optional_dynamic_size, 4)
        ;

    if (ftruncate(fd, file_size) != 0) {
        fprintf(
            stderr,
            "failed to truncate output file to size: %s\n",
            strerror(errno)
        );
        return 1;
    }

    void* data = mmap(NULL, file_size, PROT_WRITE, MAP_SHARED, fd, 0);
    if (data == (void*) -1) {
        fprintf(
            stderr,
            "failed to mmap output file: %s\n",
            strerror(errno)
        );
        return 1;
    }

    *(Elf64_Ehdr*) data = (Elf64_Ehdr) {
        .e_ident = { 0x7F, 'E', 'L', 'F', ELFCLASS64, ELFDATA2LSB, EV_CURRENT },
        .e_type = ET_REL,
        .e_machine = EM_X86_64,
        .e_version = EV_CURRENT,
        .e_entry = 0,
        .e_phoff = 0,
        .e_shoff = shtab_off,
        .e_flags = 0,
        .e_ehsize = sizeof(Elf64_Ehdr),
        .e_phentsize = 0,
        .e_phnum = 0,
        .e_shentsize = sizeof(Elf64_Shdr),
        .e_shnum = 3,
        .e_shstrndx = 1,
    };

    *(Elf64_Shdr*) (data + shstrtab_hdr_off) = (Elf64_Shdr) {
        .sh_name = 1,
        .sh_type = SHT_STRTAB,
        .sh_flags = 0,
        .sh_addr = 0,
        .sh_offset = shstrtab_data_off,
        .sh_size = shstrtab_data_size,
        .sh_link = 0,
        .sh_info = 0,
        .sh_addralign = 0,
        .sh_entsize = 0,
    };

    *(Elf64_Shdr*) (data + hk_plugin_hdr_off) = (Elf64_Shdr) {
        .sh_name = hk_plugin_name_index,
        .sh_type = SHT_NOTE,
        .sh_flags = 0,
        .sh_addr = 0,
        .sh_offset = hk_plugin_data_off,
        .sh_size = hk_plugin_get_procs_size + hk_plugin_optional_dynamic_size,
        .sh_link = 0,
        .sh_info = 0,
        .sh_addralign = 0,
        .sh_entsize = 0,
    };

    memcpy(data + shstrtab_data_off, shstrtab_data, shstrtab_data_size);

    off_t note_name_off = hk_plugin_data_off + sizeof(Elf64_Nhdr);
    off_t get_procs_symbol_off = note_name_off + note_name_size;
    *(Elf64_Nhdr*) (data + hk_plugin_data_off) = (Elf64_Nhdr) {
        .n_namesz = note_name_size,
        .n_descsz = get_procs_symbol_size,
        .n_type = HK_PLUGIN_NOTE_TYPE_GET_PROCS,
    };
    memcpy(data + note_name_off, HK_PLUGIN_NOTE_NAME, note_name_size);
    memcpy(data + get_procs_symbol_off, get_procs_symbol, get_procs_symbol_size);

    off_t note_name_off_2 = hk_plugin_optional_dynamic_off + sizeof(Elf64_Nhdr);
    if (optional_dynamic) {
        *(Elf64_Nhdr*) (data + hk_plugin_optional_dynamic_off) = (Elf64_Nhdr) {
            .n_namesz = note_name_size,
            .n_descsz = 0,
            .n_type = HK_PLUGIN_NOTE_TYPE_OPTIONAL_DYNAMIC,
        };
        memcpy(data + note_name_off_2, HK_PLUGIN_NOTE_NAME, note_name_size);
    }

    return 0;
}
