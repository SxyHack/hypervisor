/// @copyright
/// Copyright (C) 2020 Assured Information Security, Inc.
///
/// @copyright
/// Permission is hereby granted, free of charge, to any person obtaining a copy
/// of this software and associated documentation files (the "Software"), to deal
/// in the Software without restriction, including without limitation the rights
/// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
/// copies of the Software, and to permit persons to whom the Software is
/// furnished to do so, subject to the following conditions:
///
/// @copyright
/// The above copyright notice and this permission notice shall be included in
/// all copies or substantial portions of the Software.
///
/// @copyright
/// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
/// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
/// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
/// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
/// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
/// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
/// SOFTWARE.

#ifndef VM_T_HPP
#define VM_T_HPP

#include <allocated_status_t.hpp>
#include <bf_syscall_t.hpp>
#include <ept_t.hpp>
#include <gs_t.hpp>
#include <intrinsic_t.hpp>
#include <page_pool_t.hpp>
#include <tls_t.hpp>
#include <page_2m_t.hpp>
#include <map_page_flags.hpp>

#include <bsl/discard.hpp>
#include <bsl/ensures.hpp>
#include <bsl/expects.hpp>
#include <bsl/safe_integral.hpp>

namespace example
{
    /// @class example::vm_t
    ///
    /// <!-- description -->
    ///   @brief Defines the extension's notion of a VM
    ///
    class vm_t final
    {
        /// @brief stores the ID associated with this vm_t
        bsl::safe_u16 m_id{};
        /// @brief stores whether or not this vm_t is allocated.
        allocated_status_t m_allocated{};
        /// @brief stores whether or not this vm_t is active.
        bsl::array<bool, HYPERVISOR_MAX_PPS.get()> m_active{};

        extented_page_table_t m_ept{};

    public:
        /// <!-- description -->
        ///   @brief Initializes this vm_t
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param sys the bf_syscall_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///   @param i the ID for this vm_t
        ///
        constexpr void
        initialize(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t const &sys,
            intrinsic_t const &intrinsic,
            bsl::safe_u16 const &i) noexcept
        {
            bsl::discard(gs);
            bsl::discard(tls);
            bsl::discard(sys);
            bsl::discard(intrinsic);

            bsl::expects(this->id() == syscall::BF_INVALID_ID);

            bsl::expects(i.is_valid_and_checked());
            bsl::expects(i != syscall::BF_INVALID_ID);

            m_id = ~i;
        }

        /// <!-- description -->
        ///   @brief Release the vm_t.
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param sys the bf_syscall_t to use
        ///   @param mut_page_pool the page_pool_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///
        constexpr void
        release(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t const &sys,
            page_pool_t &mut_page_pool,
            intrinsic_t const &intrinsic) noexcept
        {
            this->deallocate(gs, tls, sys, mut_page_pool, intrinsic);

            m_id = {};
        }

        /// <!-- description -->
        ///   @brief Returns the ID of this vm_t
        ///
        /// <!-- inputs/outputs -->
        ///   @return Returns the ID of this vm_t
        ///
        [[nodiscard]] constexpr auto
        id() const noexcept -> bsl::safe_u16
        {
            bsl::ensures(m_id.is_valid_and_checked());
            return ~m_id;
        }

        /// <!-- description -->
        ///   @brief Allocates the vm_t and returns it's ID
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param mut_page_pool the page_pool_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///   @return Returns ID of this vm_t
        ///
        [[maybe_unused]] constexpr auto
        allocate(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t &mut_sys,
            page_pool_t &mut_page_pool,
            intrinsic_t const &intrinsic) noexcept -> bsl::safe_u16
        {
            bsl::discard(gs);
            bsl::discard(tls);
            bsl::discard(mut_page_pool);
            bsl::discard(intrinsic);

            bsl::errc_type mut_ret{};

            bsl::expects(this->id() != syscall::BF_INVALID_ID);
            bsl::expects(allocated_status_t::deallocated == m_allocated);

            mut_ret = m_ept.initialize(tls, mut_page_pool, mut_sys);
            if (bsl::unlikely(!mut_ret)) {
                bsl::print<bsl::V>() << bsl::here();
                return bsl::safe_u16::failure();
            }

            constexpr auto max_gpa{bsl::to_u64(0x8000000000U)};
            constexpr auto gpa_inc{bsl::to_idx(PAGE_2M_T_SIZE)};

            if (mut_sys.is_vm_the_root_vm(this->id())) {
                bsl::debug() << "vm "                                           // --
                             << bsl::grn << bsl::hex(this->id()) << bsl::rst    // --
                             << " was created"                                  // --
                             << bsl::endl;                                      // --
                for (bsl::safe_idx mut_i{}; mut_i < max_gpa; mut_i += gpa_inc) {
                    auto const spa{bsl::to_u64(mut_i)};
                    auto const gpa{bsl::to_u64(mut_i)};

                    mut_ret = m_ept.map<l1e_t>(tls, mut_page_pool, gpa, spa, MAP_PAGE_RWE, false, mut_sys);
                    if (bsl::unlikely(!mut_ret)) {
                        bsl::print<bsl::V>() << bsl::here();
                        return bsl::safe_u16::failure();
                    }
                    bsl::touch();
                }

                mut_page_pool.dump(tls);
                bsl::debug() << "mapped ept_phys:" << bsl::hex(m_ept.spa()) << bsl::endl;
            }
            else {
                bsl::touch();
            }

            m_allocated = allocated_status_t::allocated;
            return this->id();
        }

        /// <!-- description -->
        ///   @brief Deallocates the vm_t
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param sys the bf_syscall_t to use
        ///   @param mut_page_pool the page_pool_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///
        constexpr void
        deallocate(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t const &sys,
            page_pool_t &mut_page_pool,
            intrinsic_t const &intrinsic) noexcept
        {
            bsl::discard(gs);
            bsl::discard(intrinsic);
            bsl::discard(mut_page_pool);

            bsl::expects(this->is_active(tls).is_invalid());

            //m_emulated_mmio.deallocate(gs, tls, sys, mut_page_pool, intrinsic);
            m_allocated = allocated_status_t::deallocated;

            if (!sys.is_vm_the_root_vm(this->id())) {
                bsl::debug<bsl::V>()                                   // --
                    << "vm "                                           // --
                    << bsl::red << bsl::hex(this->id()) << bsl::rst    // --
                    << " was destroyed"                                // --
                    << bsl::endl;                                      // --
            }
            else {
                bsl::touch();
            }
        }

        /// <!-- description -->
        ///   @brief Returns true if this vm_t is allocated, false otherwise
        ///
        /// <!-- inputs/outputs -->
        ///   @return Returns true if this vm_t is allocated, false otherwise
        ///
        [[nodiscard]] constexpr auto
        is_allocated() const noexcept -> bool
        {
            return m_allocated == allocated_status_t::allocated;
        }

        /// <!-- description -->
        ///   @brief Returns true if this vm_t is deallocated, false otherwise
        ///
        /// <!-- inputs/outputs -->
        ///   @return Returns true if this vm_t is deallocated, false otherwise
        ///
        [[nodiscard]] constexpr auto
        is_deallocated() const noexcept -> bool
        {
            return m_allocated == allocated_status_t::deallocated;
        }

        /// <!-- description -->
        ///   @brief Sets this vm_t as active.
        ///
        /// <!-- inputs/outputs -->
        ///   @param mut_tls the current TLS block
        ///
        constexpr void
        set_active(tls_t &mut_tls) noexcept
        {
            auto const ppid{bsl::to_idx(mut_tls.ppid)};

            bsl::expects(allocated_status_t::allocated == m_allocated);
            bsl::expects(syscall::BF_INVALID_ID == mut_tls.active_vmid);
            bsl::expects(ppid < m_active.size());

            *m_active.at_if(ppid) = true;
            mut_tls.active_vmid = this->id();
        }

        /// <!-- description -->
        ///   @brief Sets this vm_t as inactive.
        ///
        /// <!-- inputs/outputs -->
        ///   @param mut_tls the current TLS block
        ///
        constexpr void
        set_inactive(tls_t &mut_tls) noexcept
        {
            auto const ppid{bsl::to_idx(mut_tls.ppid)};

            bsl::expects(allocated_status_t::allocated == m_allocated);
            bsl::expects(this->id() == mut_tls.active_vmid);
            bsl::expects(ppid < m_active.size());

            *m_active.at_if(ppid) = false;
            mut_tls.active_vmid = syscall::BF_INVALID_ID;
        }

        /// <!-- description -->
        ///   @brief Returns the ID of the first identified PP this vm_t is
        ///     active on. If the vm_t is not active, bsl::safe_u16::failure()
        ///     is returned.
        ///
        /// <!-- inputs/outputs -->
        ///   @param tls the current TLS block
        ///   @return Returns the ID of the first identified PP this vm_t is
        ///     active on. If the vm_t is not active, bsl::safe_u16::failure()
        ///     is returned.
        ///
        [[nodiscard]] constexpr auto
        is_active(tls_t const &tls) const noexcept -> bsl::safe_u16
        {
            auto const online_pps{bsl::to_umx(tls.online_pps)};
            bsl::expects(online_pps <= m_active.size());

            for (bsl::safe_idx mut_i{}; mut_i < online_pps; ++mut_i) {
                if (*m_active.at_if(mut_i)) {
                    return bsl::to_u16(mut_i);
                }

                bsl::touch();
            }

            return bsl::safe_u16::failure();
        }

        /// <!-- description -->
        ///   @brief Returns true if this vm_t is active on the current PP,
        ///     false otherwise
        ///
        /// <!-- inputs/outputs -->
        ///   @param tls the current TLS block
        ///   @return Returns true if this vm_t is active on the current PP,
        ///     false otherwise
        ///
        //[[nodiscard]] constexpr auto
        //is_active_on_this_pp(tls_t const &tls) const noexcept -> bool
        //{
        //    bsl::expects(bsl::to_umx(tls.ppid) < m_active.size());
        //    return *m_active.at_if(bsl::to_idx(tls.ppid));
        //}

        /// <!-- description -->
        ///   @brief Returns the system physical address of the second level
        ///     page tables used by this vm_t.
        ///
        /// <!-- inputs/outputs -->
        ///   @return Returns the system physical address of the second level
        ///     page tables used by this vm_t.
        ///
        [[nodiscard]] constexpr auto
        eptp() const noexcept -> bsl::safe_u64
        {
            return m_ept.spa();
        }

        /// <!-- description -->
        ///   @brief Maps memory into this vm_t using instructions from the
        ///     provided MDL.
        ///
        /// <!-- inputs/outputs -->
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param mut_page_pool the page_pool_t to use
        ///   @param mdl the MDL containing the memory to map into the vm_t
        ///   @return Returns bsl::errc_success on success, bsl::errc_failure
        ///     and friends otherwise
        ///
        //[[nodiscard]] constexpr auto
        //mmio_map(
        //    tls_t const &tls,
        //    syscall::bf_syscall_t &mut_sys,
        //    page_pool_t &mut_page_pool,
        //    hypercall::mv_mdl_t const &mdl) noexcept -> bsl::errc_type
        //{
        //    return m_emulated_mmio.map(tls, mut_sys, mut_page_pool, mdl);
        //}

        /// <!-- description -->
        ///   @brief Unmaps memory from this vm_t using instructions from the
        ///     provided MDL.
        ///
        /// <!-- inputs/outputs -->
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param mut_page_pool the page_pool_t to use
        ///   @param mdl the MDL containing the memory to map from the vm_t
        ///   @return Returns bsl::errc_success on success, bsl::errc_failure
        ///     and friends otherwise
        ///
        //[[nodiscard]] constexpr auto
        //mmio_unmap(
        //    tls_t const &tls,
        //    syscall::bf_syscall_t &mut_sys,
        //    page_pool_t &mut_page_pool,
        //    hypercall::mv_mdl_t const &mdl) noexcept -> bsl::errc_type
        //{
        //    return m_emulated_mmio.unmap(tls, mut_sys, mut_page_pool, mdl);
        //}

        /// <!-- description -->
        ///   @brief Returns a system physical address given a guest physical
        ///     address using MMIO second level paging from this vm_t to
        ///     perform the translation.
        ///
        /// <!-- inputs/outputs -->
        ///   @param sys the bf_syscall_t to use
        ///   @param gpa the GPA to translate to a SPA
        ///   @return Returns a system physical address given a guest physical
        ///     address using MMIO second level paging from this vm_t to
        ///     perform the translation.
        ///
        //[[nodiscard]] constexpr auto
        //gpa_to_spa(syscall::bf_syscall_t const &sys, bsl::safe_u64 const &gpa) const noexcept
        //    -> bsl::safe_u64
        //{
        //    bsl::expects(allocated_status_t::allocated == m_allocated);
        //    return m_emulated_mmio.gpa_to_spa(sys, gpa);
        //}
    };
}

#endif
