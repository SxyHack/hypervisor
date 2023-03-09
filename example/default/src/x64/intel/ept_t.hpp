#pragma once

#include <basic_root_page_table_t.hpp>
#include <bf_syscall_t.hpp>
#include <intrinsic_t.hpp>
#include <tls_t.hpp>
#include <l0e_t.hpp>
#include <l1e_t.hpp>
#include <l2e_t.hpp>
#include <l3e_t.hpp>
#include <page_pool_t.hpp>

namespace example
{
    /// @brief defines the extented_page_table_t used by the microkernel
    using extented_page_table_t = lib::basic_root_page_table_t<
        tls_t, 
        syscall::bf_syscall_t, 
        page_pool_t, 
        intrinsic_t, 
        l3e_t, 
        l2e_t, 
        l1e_t, 
        l0e_t>;
}
