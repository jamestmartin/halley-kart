#define _POSIX_C_SOURCE 200112L

#include <elf.h>
#include <errno.h>
#include <fcntl.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#ifndef align_up
#define align_up(num, align) (((num) + ((align)-1)) & ~((align)-1))
#endif

static const char* const usage =
    "usage: hk-make-elf-blob <output file> <section name> <symbol name> <blob file>\n";

int main(int argc, char** argv) {
    if (argc < 5) {
        fprintf(stderr, "not enough arguments\n%s", usage);
        return 1;
    }
    if (argc > 5) {
        fprintf(stderr, "too many arguments\n%s", usage);
    }

    char* output_file_name = argv[1];
    char* section_name = argv[2];
    size_t section_name_size = strlen(section_name) + 1;
    char* symbol_name = argv[3];
    size_t symbol_name_size = strlen(symbol_name) + 1;
    char* blob_file_name = argv[4];

    int blob_fd = open(blob_file_name, O_RDONLY);
    if (blob_fd < 0) {
        fprintf(
            stderr,
            "failed to open blob file %s: %s\n%s",
            output_file_name,
            strerror(errno),
            usage
        );
        return 1;
    }

    off_t blob_size = lseek(blob_fd, 0, SEEK_END);
    if (blob_size == (off_t) -1) {
        fprintf(
            stderr,
            "failed to get size of blob file: %s\n",
            strerror(errno)
        );
        return 1;
    }

    void* blob =
        mmap((void*) -1, blob_size, PROT_READ, MAP_PRIVATE, blob_fd, 0);
    if (blob == (void*) -1) {
        fprintf(stderr, "failed to mmap blob file: %s\n", strerror(errno));
        return 1;
    }

    close(blob_fd);

    // only write is needed, but mmap requires O_RDWR
    int output_fd = open(output_file_name, O_RDWR | O_CREAT | O_TRUNC);
    if (output_fd < 0) {
        fprintf(
            stderr,
            "failed to open output file %s: %s\n%s",
            output_file_name,
            strerror(errno),
            usage
        );
        return 1;
    }

    char* shstrtab_fixed_data = "\0.shstrtab\0.strtab\0.symtab\0";
    size_t shstrtab_shstrtab_off = 1;
    size_t shstrtab_strtab_off = 11;
    size_t shstrtab_symtab_off = 19;
    size_t shstrtab_blob_off = 28;
    size_t shstrtab_fixed_data_size = 28;
    size_t shstrtab_data_size = shstrtab_fixed_data_size + section_name_size;

    size_t strtab_name_off = 1;
    size_t strtab_size_off = strtab_name_off + symbol_name_size;
    size_t strtab_end_off = strtab_size_off + symbol_name_size + 5;

    size_t symtab_data_size = 3 * sizeof(Elf64_Sym);

    Elf64_Off ehdr_off = 0;
    Elf64_Off shtab_off = ehdr_off + sizeof(Elf64_Ehdr);
    Elf64_Off shstrtab_hdr_off = shtab_off + sizeof(Elf64_Shdr);
    Elf64_Off strtab_hdr_off = shstrtab_hdr_off + sizeof(Elf64_Shdr);
    Elf64_Off symtab_hdr_off = strtab_hdr_off + sizeof(Elf64_Shdr);
    Elf64_Off blob_hdr_off = symtab_hdr_off + sizeof(Elf64_Shdr);
    Elf64_Off shstrtab_data_off = blob_hdr_off + sizeof(Elf64_Shdr);
    Elf64_Off strtab_data_off = shstrtab_data_off + shstrtab_data_size;
    Elf64_Off strtab_data_name_off = strtab_data_off + strtab_name_off;
    Elf64_Off strtab_data_size_off = strtab_data_off + strtab_size_off;
    Elf64_Off symtab_data_off = strtab_data_off + strtab_end_off;
    Elf64_Off blob_data_off = symtab_data_off + symtab_data_size + 8;
    Elf64_Off file_end_off = blob_data_off + blob_size;

    if (ftruncate(output_fd, file_end_off) != 0) {
        fprintf(
            stderr,
            "failed to truncate output file to size: %s\n",
            strerror(errno)
        );
        return 1;
    }

    void* data =
        mmap((void*) -1, file_end_off, PROT_WRITE, MAP_SHARED, output_fd, 0);
    if (data == (void*) -1) {
        fprintf(
            stderr,
            "failed to mmap output file: %s\n",
            strerror(errno)
        );
        return 1;
    }

    close(output_fd);

    *(Elf64_Ehdr*) (data + ehdr_off) = (Elf64_Ehdr) {
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
        .e_shnum = 5,
        .e_shstrndx = 1,
    };

    *(Elf64_Shdr*) (data + shstrtab_hdr_off) = (Elf64_Shdr) {
        .sh_name = shstrtab_shstrtab_off,
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

    *(Elf64_Shdr*) (data + strtab_hdr_off) = (Elf64_Shdr) {
        .sh_name = shstrtab_strtab_off,
        .sh_type = SHT_STRTAB,
        .sh_flags = 0,
        .sh_addr = 0,
        .sh_offset = strtab_data_off,
        .sh_size = strtab_end_off,
        .sh_link = 0,
        .sh_info = 0,
        .sh_addralign = 0,
        .sh_entsize = 0,
    };

    *(Elf64_Shdr*) (data + symtab_hdr_off) = (Elf64_Shdr) {
        .sh_name = shstrtab_symtab_off,
        .sh_type = SHT_SYMTAB,
        .sh_flags = 0,
        .sh_addr = 0,
        .sh_offset = symtab_data_off,
        .sh_size = symtab_data_size,
        .sh_link = 2,
        .sh_info = 1,
        .sh_addralign = 0,
        .sh_entsize = sizeof(Elf64_Sym),
    };

    *(Elf64_Shdr*) (data + blob_hdr_off) = (Elf64_Shdr) {
        .sh_name = shstrtab_blob_off,
        .sh_type = SHT_PROGBITS,
        .sh_flags = SHF_ALLOC,
        .sh_addr = 0,
        .sh_offset = blob_data_off,
        .sh_size = blob_size,
        .sh_link = 0,
        .sh_info = 0,
        .sh_addralign = 0,
        .sh_entsize = 0,
    };

    memcpy(
        data + shstrtab_data_off,
        shstrtab_fixed_data,
        shstrtab_fixed_data_size
    );
    memcpy(
        data + shstrtab_data_off + shstrtab_fixed_data_size,
        section_name,
        section_name_size
    );

    memcpy(data + strtab_data_name_off, symbol_name, symbol_name_size);
    memcpy(
        data + strtab_data_size_off,
        symbol_name,
        symbol_name_size - 1
    );
    memcpy(
        data + strtab_data_size_off + symbol_name_size - 1,
        "_size",
        5
    );

    *(Elf64_Sym*) (data + symtab_data_off + sizeof(Elf64_Sym)) = (Elf64_Sym) {
        .st_name = strtab_name_off,
        .st_info = ELF64_ST_INFO(STB_GLOBAL, STT_OBJECT),
        .st_other = ELF64_ST_VISIBILITY(STV_DEFAULT),
        .st_shndx = 4,
        .st_value = 8,
        .st_size = blob_size,
    };

    // I wanted to make this an absolute symbol,
    // but it segfaulted for whatever reason, probably relocation-related.
    *(Elf64_Sym*) (data + symtab_data_off + 2 * sizeof(Elf64_Sym)) = (Elf64_Sym) {
        .st_name = strtab_size_off,
        .st_info = ELF64_ST_INFO(STB_GLOBAL, STT_OBJECT),
        .st_other = ELF64_ST_VISIBILITY(STV_DEFAULT),
        .st_shndx = 4,
        .st_value = 0,
        .st_size = 8,
    };

    *(uint64_t*) (data + blob_data_off) = blob_size;
    memcpy(data + blob_data_off + 8, blob, blob_size);

    return 0;
}
