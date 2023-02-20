#pragma once

#include <types.h>

#ifdef __cplusplus
extern "C"
{
#endif

#pragma pack(push, 1)

#ifdef _MSC_VER
#pragma warning(disable : 4214)
#endif

    /**
     * <!-- description -->
     *   @brief Defines the layout of a page table entry (PTE).
     */
    union ept_pt_entry_t
    {
        uint64_t all;
        struct
        {
            // [Bit 0] Read access; indicates whether reads are allowed from the 4-KByte page referenced by this entry
            uint64_t read_access : 1;
            // [Bit 1] Write access;
            uint64_t write_access : 1;
            // [Bit 2] If the "mode-based execute control for EPT" VM-execution control is 0, execute access;
            uint64_t exec_access : 1;
            // [Bits 5:3] EPT memory type for this 4-KByte page
            uint64_t memory_type : 3;
            // [Bit 6] Ignore PAT memory type for this 4-KByte page
            uint64_t ignore_pat : 1;
            uint64_t reserved_1 : 1;
            // [Bit 8] If bit 6 of EPTP is 1, accessed flag for EPT
            uint64_t accessed : 1;
            // [Bit 9] If bit 6 of EPTP is 1, dirty flag for EPT
            uint64_t dirty : 1;
            // [Bit 10] Execute access for user-mode linear addresses.
            uint64_t user_mode_execute : 1;
            uint64_t reserved_2 : 1;
            // [Bits 47:12] Physical address of 4-KByte aligned EPT page-directory-pointer table referenced by this entry.
            uint64_t page_frame_number : 36;
            uint64_t reserved_3 : 15;
            // [Bit 63] Suppress \#VE. @see Vol3C[25.5.6.1(Convertible EPT Violations)]
            uint64_t suppress_ve : 1;
        };
    };

#ifdef _MSC_VER
#pragma warning(default : 4214)
#endif

#pragma pack(pop)

#ifdef __cplusplus
}
#endif