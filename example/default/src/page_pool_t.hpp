#pragma once

#include "page_pool_helpers.hpp"

#include <basic_page_pool_t.hpp>
#include <bf_syscall_t.hpp>
#include <tls_t.hpp>

namespace example
{
    /// @brief defines the page_pool_t used by the microkernel
    using page_pool_t = lib::basic_page_pool_t<
        tls_t, 
        syscall::bf_syscall_t, 
        HYPERVISOR_EXT_PAGE_POOL_ADDR.get(), 
        HYPERVISOR_EXT_DIRECT_MAP_SIZE.get()>;
}
