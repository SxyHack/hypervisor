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

const EXIT_REASON_NMI: u64 = 0x0;
const EXIT_REASON_NMI_WINDOW: u64 = 0x8;
const EXIT_REASON_CPUID: u64 = 0xA;
const EXIT_REASON_EPT_VIOLATION: u64 = 48;
const EXIT_REASON_EPT_MISCONFIG: u64 = 49;

#[path = "dispatch_vmexit_nmi_window.rs"]
#[doc(hidden)]
pub mod dispatch_vmexit_nmi_window;
pub use dispatch_vmexit_nmi_window::*;

#[path = "dispatch_vmexit_nmi.rs"]
#[doc(hidden)]
pub mod dispatch_vmexit_nmi;
pub use dispatch_vmexit_nmi::*;

#[path = "../dispatch_vmexit_cpuid.rs"]
#[doc(hidden)]
pub mod dispatch_vmexit_cpuid;
pub use dispatch_vmexit_cpuid::*;

#[path = "dispatch_vmexit_ept.rs"]
#[doc(hidden)]
pub mod dispatch_vmexit_ept;
pub use dispatch_vmexit_ept::*;

/// <!-- description -->
///   @brief Dispatches the VMExit.
///
/// <!-- inputs/outputs -->
///   @param gs the gs_t to use
///   @param tls the tls_t to use
///   @param sys the bf_syscall_t to use
///   @param intrinsic the intrinsic_t to use
///   @param vp_pool the vp_pool_t to use
///   @param vs_pool the vs_pool_t to use
///   @param vsid the ID of the VS that generated the VMExit
///   @param exit_reason the exit reason associated with the VMExit
///   @return Returns bsl::errc_success on success, bsl::errc_failure
///     and friends otherwise
///
pub fn dispatch_vmexit(
    gs: &crate::GsT,
    tls: &crate::TlsT,
    sys: &mut syscall::BfSyscallT,
    intrinsic: &crate::IntrinsicT,
    vp_pool: &crate::VpPoolT,
    vs_pool: &crate::VsPoolT,
    vsid: bsl::SafeU16,
    exit_reason: bsl::SafeU64,
) -> bsl::ErrcType {
    bsl::discard(vp_pool);
    bsl::discard(vs_pool);

    match exit_reason.get() {
        EXIT_REASON_NMI => return dispatch_vmexit_nmi(gs, tls, sys, vsid),
        EXIT_REASON_NMI_WINDOW => return dispatch_vmexit_nmi_window(gs, tls, sys, vsid),
        EXIT_REASON_CPUID => return dispatch_vmexit_cpuid(gs, tls, sys, intrinsic, vsid),
        EXIT_REASON_EPT_VIOLATION => return dispatch_vmexit_ept_violation(gs, tls, sys, intrinsic, vsid),
        EXIT_REASON_EPT_MISCONFIG => return dispatch_vmexit_ept_misconfig(gs, tls, sys, intrinsic, vsid),
        _ => {}
    }

    error!("unsupported vmexit: {:#018x}\n", exit_reason);
    syscall::bf_debug_op_dump_vs(vsid);
    print_v!("{}", bsl::here());

    return bsl::errc_failure;
}
