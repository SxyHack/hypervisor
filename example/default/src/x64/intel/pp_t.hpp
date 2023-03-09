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

#ifndef PP_T_HPP
#define PP_T_HPP

#include <allocated_status_t.hpp>
#include <bf_syscall_t.hpp>
#include <gs_t.hpp>
#include <intrinsic_t.hpp>
#include <tls_t.hpp>

#include <bsl/discard.hpp>
#include <bsl/ensures.hpp>
#include <bsl/expects.hpp>
#include <bsl/safe_integral.hpp>

namespace example
{
    /// @class example::pp_t
    ///
    /// <!-- description -->
    ///   @brief Defines the extension's notion of a PP
    ///
    class pp_t final
    {
        /// @brief stores the ID associated with this pp_t
        bsl::safe_u16 m_id{};

    public:
        /// <!-- description -->
        ///   @brief Initializes this pp_t
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///   @param i the ID for this pp_t
        ///
        constexpr void
        initialize(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t &mut_sys,
            intrinsic_t const &intrinsic,
            bsl::safe_u16 const &i) noexcept
        {
            bsl::discard(gs);
            bsl::discard(tls);
            bsl::discard(mut_sys);
            bsl::discard(intrinsic);
            
            bsl::expects(this->id() == syscall::BF_INVALID_ID);

            m_id = ~i;
        }

        /// <!-- description -->
        ///   @brief Release the pp_t.
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param mut_sys the bf_syscall_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///
        constexpr void
        release(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t &mut_sys,
            intrinsic_t const &intrinsic) noexcept
        {
            bsl::discard(gs);
            bsl::discard(tls);
            bsl::discard(mut_sys);
            bsl::discard(intrinsic);

            m_id = {};
        }

        /// <!-- description -->
        ///   @brief Returns the ID of this pp_t
        ///
        /// <!-- inputs/outputs -->
        ///   @return Returns the ID of this pp_t
        ///
        [[nodiscard]] constexpr auto
        id() const noexcept -> bsl::safe_u16
        {
            bsl::ensures(m_id.is_valid_and_checked());
            return ~m_id;
        }

        /// <!-- description -->
        ///   @brief Allocates the pp_t and returns it's ID
        ///
        /// <!-- inputs/outputs -->
        ///   @param gs the gs_t to use
        ///   @param tls the tls_t to use
        ///   @param sys the bf_syscall_t to use
        ///   @param page_pool the page_pool_t to use
        ///   @param intrinsic the intrinsic_t to use
        ///   @return Returns ID of this pp_t
        ///
        [[nodiscard]] constexpr auto
        allocate(
            gs_t const &gs,
            tls_t const &tls,
            syscall::bf_syscall_t const &sys,
            page_pool_t const &page_pool,
            intrinsic_t const &intrinsic) const noexcept
        {
            bsl::expects(this->id() != syscall::BF_INVALID_ID);

            bsl::discard(gs);
            bsl::discard(tls);
            bsl::discard(sys);
            bsl::discard(page_pool);
            bsl::discard(intrinsic);

            /// TODO:
            /// - We need to detect all of the features that we need
            ///   support for here and error out if the CPU does not
            ///   support the features that we need. This includes
            ///
            ///   - CPUID
            ///   - MSRs
            ///
                
        }
    };
}

#endif
