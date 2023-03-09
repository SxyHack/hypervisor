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

#ifndef PP_POOL_T_HPP
#define PP_POOL_T_HPP

#include <bf_syscall_t.hpp>
#include <gs_t.hpp>
#include <page_pool_t.hpp>
#include <pp_t.hpp>
#include <tls_t.hpp>

#include <bsl/array.hpp>
#include <bsl/convert.hpp>
#include <bsl/debug.hpp>
#include <bsl/expects.hpp>
#include <bsl/safe_idx.hpp>
#include <bsl/safe_integral.hpp>
#include <bsl/touch.hpp>
#include <bsl/unlikely.hpp>

namespace example
{
    /// @class example::pp_pool_t
    ///
    /// <!-- description -->
    ///   @brief Defines the extension's PP pool
    ///
    class pp_pool_t final
    {
        /// @brief stores the pool of pp_t objects
        bsl::array<pp_t, HYPERVISOR_MAX_PPS.get()> m_pool{};

        /// <!-- description -->
        ///   @brief Returns the pp_t associated with the provided ppid.
        ///
        /// <!-- inputs/outputs -->
        ///   @param ppid the ID of the pp_t to get
        ///   @return Returns the pp_t associated with the provided ppid.
        ///
        [[nodiscard]] constexpr auto
        get_pp(bsl::safe_u16 const &ppid) noexcept -> pp_t *
        {
            bsl::expects(ppid.is_valid_and_checked());
            bsl::expects(ppid < bsl::to_u16(m_pool.size()));
            return m_pool.at_if(bsl::to_idx(ppid));
        }

        /// <!-- description -->
        ///   @brief Returns the pp_t associated with the provided ppid.
        ///
        /// <!-- inputs/outputs -->
        ///   @param ppid the ID of the pp_t to get
        ///   @return Returns the pp_t associated with the provided ppid.
        ///
        [[nodiscard]] constexpr auto
        get_pp(bsl::safe_u16 const &ppid) const noexcept -> pp_t const *
        {
            bsl::expects(ppid.is_valid_and_checked());
            bsl::expects(ppid < bsl::to_u16(m_pool.size()));
            return m_pool.at_if(bsl::to_idx(ppid));
        }

    public:
        /// <!-- description -->
        ///   @brief Initializes this pp_pool_t
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///
        constexpr void
        initialize(
            gs_t const &gs, tls_t const &tls, syscall::bf_syscall_t &mut_sys, intrinsic_t const &intrinsic) noexcept
        {
            for (bsl::safe_idx mut_i{}; mut_i < m_pool.size(); ++mut_i) {
                m_pool.at_if(mut_i)->initialize(gs, tls, mut_sys, intrinsic, bsl::to_u16(mut_i));
            }
        }

        /// <!-- description -->
        ///   @brief Release the pp_pool_t.
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///
        constexpr void
        release(gs_t const &gs, tls_t const &tls, syscall::bf_syscall_t &mut_sys, intrinsic_t const &intrinsic) noexcept
        {
            for (auto &mut_pp : m_pool) {
                mut_pp.release(gs, tls, mut_sys, intrinsic);
            }
        }

        /// <!-- description -->
        ///   @brief Allocates a PP
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param page_pool the page_pool_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///   @param ppid to use
        ///
        [[nodiscard]] constexpr auto
        allocate(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t &mut_sys,
            intrinsic_t const &intrinsic,
            page_pool_t const &page_pool,
            bsl::safe_u16 const &ppid) noexcept
        {
            this->get_pp(ppid)->allocate(gs, tls, mut_sys, page_pool, intrinsic);
        }
    };
}

#endif
