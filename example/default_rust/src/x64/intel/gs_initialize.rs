use crate::{map_2m_page, ExtentPageTable, mtrr_t::MtrrT};

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

/// <!-- description -->
///   @brief Initializes the Global Storage (GS).
///
/// <!-- inputs/outputs -->
///   @param gs the gs_t to use
///   @param sys the bf_syscall_t to use
///   @param intrinsic the intrinsic_t to use
///   @return Returns bsl::errc_success on success, bsl::errc_failure
///     and friends otherwise
///
pub fn gs_initialize(
    gs: &mut crate::GsT,
    sys: &syscall::BfSyscallT,
    intrinsic: &crate::IntrinsicT,
) -> bsl::ErrcType {
    bsl::discard(intrinsic);

    gs.msr_bitmap = sys.bf_mem_op_alloc_page::<u8>(&mut gs.msr_bitmap_phys);
    if core::ptr::null_mut() == gs.msr_bitmap {
        print_v!("{}", bsl::here());
        return bsl::errc_failure;
    }

    bsl::debug!("Allocated MSR Bitmap: {:#08X}\n", gs.msr_bitmap_phys);

    let ret = gs.mtrr.initialize(sys, intrinsic);
    if !ret.success() {
        bsl::error!("{}", bsl::here());
        return bsl::errc_failure;
    }

    let virt = core::ptr::addr_of!(gs.mtrr);
    bsl::debug!("mtrr={:#016X}\n", virt.addr());

    let size = bsl::to_u64(core::mem::size_of::<ExtentPageTable>());
    bsl::debug!("ept size: {:#06x}\n", size);
    gs.ept = sys.bf_mem_op_alloc_huge(size, &mut gs.ept_phys);
    bsl::debug!(
        "ept addr: phys={:#018x} virt={:#018x}\n",
        gs.ept_phys,
        gs.ept.addr()
    );
    if core::ptr::null_mut() == gs.ept {
        bsl::error!("allocated EPT failed, {}\n", bsl::here());
        return bsl::errc_failure;
    }

    // build_ept_map(sys, intrinsic, gs)
    bsl::errc_success
}

fn build_ept_map(
    sys: &syscall::BfSyscallT,
    intrinsic: &crate::IntrinsicT,
    gs: &mut crate::GsT,
) -> bsl::ErrcType {
    // bsl::discard(sys);
    // bsl::discard(gs);

    let page_2m = bsl::SafeU64::new(0x200000);
    let page_4k = syscall::HYPERVISOR_PAGE_SIZE;
    let mut cursor = bsl::SafeU64::new(0);
    let mut count = 0;
    while cursor < gs.mtrr.max_phys {
        let memory_type = gs.mtrr.get_memory_type(cursor);
        let ept = unsafe { gs.ept.as_mut().unwrap() };
        let ret = map_2m_page(sys, intrinsic, ept, cursor, memory_type);
        if !ret.success() {
            bsl::error!("alloc_2m_page failed, {}", bsl::here());
            return bsl::errc_failure;
        }

        cursor += page_4k;
        count += 1;
    }

    bsl::debug!(
        "build ept map({:#06x}), {:#018x} == {:#018x}\n",
        count,
        cursor,
        gs.mtrr.max_phys
    );

    syscall::bf_debug_op_dump_huge_pool();
    bsl::errc_success
}
