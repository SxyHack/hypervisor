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
     *   @brief Defines the layout of a EPT Pointer.
     */
    union ept_pointer_t
    {
        uint64_t all;
        struct
        {
            // [Bit 0] Read access
            uint64_t read_access : 1;
            // [Bit 1] Write access
            uint64_t write_access : 1;
            // [Bit 2] execute access
            uint64_t exec_access : 1;
            uint64_t reserved1 : 5;
            // [Bit 8] If bit 6 of EPTP is 1, accessed flag for EPT. @see Vol3C[28.2.4(Accessed and Dirty Flags for EPT)]
            uint64_t accessed : 1;
            uint64_t reserved2 : 1;
            // [Bit 10] Execute access for user-mode linear addresses.
            uint64_t user_mode_exec : 1;
            uint64_t reserved3 : 1;
            // [Bits 47:12] Physical address of 4-KByte aligned EPT page-directory-pointer table referenced by this entry.
            uint64_t page_frame_number : 36;
            uint64_t reserved4 : 16;
        };
    };

#ifdef _MSC_VER
#pragma warning(default : 4214)
#endif

#pragma pack(pop)

#ifdef __cplusplus
}
#endif