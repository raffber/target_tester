#include <string>
#include <vector>
#include <iostream>
#include "link.h"

/* Callback for dl_iterate_phdr.
 * Is called by dl_iterate_phdr for every loaded shared lib until something
 * else than 0 is returned by one call of this function.
 */
int retrieve_symbolnames(struct dl_phdr_info *info, size_t info_size, void *symbol_names_vector) {

    /* ElfW is a macro that creates proper typenames for the used system architecture
     * (e.g. on a 32 bit system, ElfW(Dyn*) becomes "Elf32_Dyn*") */
    ElfW(Dyn *)dyn;
    ElfW(Sym *)sym;
    ElfW(Word *)hash;

    char *strtab = 0;
    char *sym_name = 0;
    ElfW(Word) sym_cnt = 0;

    /* the void pointer (3rd argument) should be a pointer to a vector<string>
     * in this example -> cast it to make it usable */
    auto symbol_names = reinterpret_cast<std::vector<std::string> *>(symbol_names_vector);

    /* Iterate over all headers of the current shared lib
     * (first call is for the executable itself) */
    for (size_t header_index = 0; header_index < info->dlpi_phnum; header_index++) {

        /* Get a pointer to the first entry of the dynamic section.
         * It's address is the shared lib's address + the virtual address */
        dyn = (ElfW(Dyn) *) (info->dlpi_addr + info->dlpi_phdr[header_index].p_vaddr);

        /* Iterate over all entries of the dynamic section until the
         * end of the symbol table is reached. This is indicated by
         * an entry with d_tag == DT_NULL.
         *
         * Only the following entries need to be processed to find the
         * symbol names:
         *  - DT_HASH   -> second word of the hash is the number of symbols
         *  - DT_STRTAB -> pointer to the beginning of a string table that
         *                 contains the symbol names
         *  - DT_SYMTAB -> pointer to the beginning of the symbols table
         */
        while (dyn->d_tag != DT_NULL) {
            if (dyn->d_tag == DT_GNU_HASH) {
                /* Get a pointer to the hash */
                hash = (ElfW(Word *)) dyn->d_un.d_ptr;

                /* The 2nd word is the number of symbols */
                sym_cnt = hash[1];

            } else if (dyn->d_tag == DT_STRTAB) {
                /* Get the pointer to the string table */
                strtab = (char *) dyn->d_un.d_ptr;
            } else if (dyn->d_tag == DT_SYMTAB) {
                /* Get the pointer to the first entry of the symbol table */
                sym = (ElfW(Sym *)) dyn->d_un.d_ptr;


                /* Iterate over the symbol table */
                for (ElfW(Word) sym_index = 0; sym_index < sym_cnt; sym_index++) {
                    /* get the name of the i-th symbol.
                     * This is located at the address of st_name
                     * relative to the beginning of the string table. */
                    sym_name = &strtab[sym[sym_index].st_name];

                    symbol_names->push_back(std::string(sym_name));
                }
            }

            /* move pointer to the next entry */
            dyn++;
        }
    }

    /* Returning something != 0 stops further iterations,
     * since only the first entry, which is the executable itself, is needed
     * 1 is returned after processing the first entry.
     *
     * If the symbols of all loaded dynamic libs shall be found,
     * the return value has to be changed to 0.
     */
    return 1;
}

int main() {
    std::vector<std::string> symbolNames;
    dl_iterate_phdr(retrieve_symbolnames, &symbolNames);

    for (const auto &x: symbolNames) {
        std::cout << x << std::endl;
    }

    return 0;
}