pub mod ept_entry;
pub mod ept_pd_2m;
pub mod ept_pointer;

use ept_entry::*;
use ept_pointer::*;

use crate::ept_pd_2m::EptMapPd;

pub type ExtentPageTable = EptEntryPml4;

pub fn map_2m_page(
    sys: &syscall::BfSyscallT,
    intrinsic: &crate::IntrinsicT,
    ept: &mut ExtentPageTable,
    phys: bsl::SafeU64,
    memory_type: u8
) -> bsl::ErrcType {
    bsl::discard(intrinsic);

    let pml4_index = get_pml4_index(phys);
    if !pml4_index.is_zero() {
        return bsl::errc_failure;
    }

    let mut pdpt_virt = ept.tables[0];
    if pdpt_virt == 0 {
        pdpt_virt = alloc_pdpt(sys, ept, 0);
        if pdpt_virt == 0 {
            bsl::error!("{}", bsl::here());
            return bsl::errc_failure;
        }
    }

    let pdpt = core::ptr::from_exposed_addr_mut::<EptEntryPdpt>(pdpt_virt);
    let pdpt_index = get_pml3_index(phys);
    let pdpt_index = pdpt_index.get() as usize;
    // bsl::debug!("pdpt_index={}", pdpt_index);

    let pdpt = unsafe { &mut *pdpt };
    let mut pdt_virt = pdpt.tables[pdpt_index];
    if pdt_virt == 0 {
        pdt_virt = alloc_pdt(sys, pdpt, phys);
        if pdt_virt == 0 {
            bsl::error!("{}", bsl::here());
            return bsl::errc_failure;
        }
    }

    let pdt_index = get_pml2_index(phys);
    let pdt_index = pdt_index.get() as usize;

    let pdt = core::ptr::from_exposed_addr_mut::<EptEntryPdt>(pdt_virt);
    let pdt = unsafe { &mut *pdt };
    let pdt_val = pdt.entries[pdt_index];
    let mut pdt_entry = EptMapPd::from(pdt_val);

    // let phys_bits = phys_addr_bits - 21;
    // let phys_mask = bsl::SafeU64::magic_1() << phys_bits - 1;
    let phys_pfn = (phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
    // let phys_valid = (phys_pfn & phys_mask).get() as usize;

    pdt_entry.set_r(true);
    pdt_entry.set_w(true);
    pdt_entry.set_e(true);
    pdt_entry.set_is_map(true);
    pdt_entry.set_memory_type(memory_type);
    pdt_entry.set_phys(phys_pfn);

    let pdt_val: u64 = pdt_entry.into();
    pdt.entries[pdt_index] = pdt_val;
    // bsl::debug!(
    //     "{} {} {}\n",
    //     pml4_index,
    //     pdpt_index,
    //     pdt_index
    // );

    bsl::errc_success
}

fn alloc_pdpt(sys: &syscall::BfSyscallT, ept: &mut ExtentPageTable, pml4_index: usize) -> usize {
    let size = core::mem::size_of::<EptEntryPdpt>();
    let size = bsl::to_u64(size);
    let mut pml4_entry = ept.entries[pml4_index];
    let mut pdpt_phys = bsl::SafeU64::new(0);
    let pdpt_virt = sys.bf_mem_op_alloc_huge::<EptEntryPdpt>(size, &mut pdpt_phys);
    if pdpt_virt == core::ptr::null_mut() {
        bsl::error!("alloc pdpt failed. {}", bsl::here());
        return 0;
    }
    ept.tables[pml4_index] = pdpt_virt.addr();

    let pdpt_pfn = pdpt_phys >> syscall::HYPERVISOR_PAGE_SHIFT;
    pml4_entry.set_r(true);
    pml4_entry.set_w(true);
    pml4_entry.set_e(true);
    pml4_entry.set_phys(pdpt_pfn.get() as usize);

    // let pdpt_entry_hex: u64 = pdpt_entry.into();
    ept.entries[pml4_index] = pml4_entry;

    bsl::debug!(
        "alloc pdpt({:#06x}), virt={:#010x}, phys={:#010x} hex={:?}\n",
        size,
        ept.tables[pml4_index],
        pdpt_phys,
        ept.entries[pml4_index],
    );

    pdpt_virt.addr()
}

fn alloc_pdt(sys: &syscall::BfSyscallT, pdpt: &mut EptEntryPdpt, phys: bsl::SafeU64) -> usize {
    let size = core::mem::size_of::<EptEntryPdt>();
    let size = bsl::to_u64(size);
    let pdpt_index = get_pml3_index(phys);
    let pdpt_index = pdpt_index.get() as usize;
    let mut pdpt_entry = pdpt.entries[pdpt_index];

    // alloc pdt
    let mut pdt_phys = bsl::SafeU64::new(0);
    let pdt_virt = sys.bf_mem_op_alloc_huge::<EptEntryPdt>(size, &mut pdt_phys);
    if pdt_virt == core::ptr::null_mut() {
        bsl::error!("alloc pdt failed. {}", bsl::here());
        return 0;
    }
    pdpt.tables[pdpt_index] = pdt_virt.addr();

    let pdt_pfn = pdt_phys >> syscall::HYPERVISOR_PAGE_SHIFT;
    pdpt_entry.set_r(true);
    pdpt_entry.set_w(true);
    pdpt_entry.set_e(true);
    pdpt_entry.set_phys(pdt_pfn.get() as usize);

    pdpt.entries[pdpt_index] = pdpt_entry;

    // bsl::debug!(
    //     "alloc pdt-{}({:#06x}), virt={:#010x}, phys={:#010x} hex={:?}\n",
    //     pdpt_index,
    //     size,
    //     pdpt.tables[pdpt_index],
    //     pdt_phys,
    //     pdpt.entries[pdpt_index],
    // );

    pdt_virt.addr()
}

pub fn eptp(gs: &crate::GsT) -> bsl::SafeU64 {
    let pml4_pfn = (gs.ept_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
    let ret = EptPointerT::new()
        .with_memory_type(6)
        .with_enable_access_and_drity(false)
        .with_page_walk_len(3)
        .with_pfn(pml4_pfn);

    let val:u64 = ret.into();
    bsl::to_u64(val)
}

// 返回物理地址的EPML4的偏移
fn get_pml4_index(phys: bsl::SafeU64) -> bsl::SafeU64 {
    let mask = bsl::to_u64(0x1FF);
    let shift = bsl::to_u64(39);
    (phys >> shift) & mask
}

// 返回物理地址的EPDPT的偏移
fn get_pml3_index(phys: bsl::SafeU64) -> bsl::SafeU64 {
    let mask = bsl::to_u64(0x1FF);
    let shift = bsl::to_u64(30);
    (phys >> shift) & mask
}

// 返回物理地址的EPDT的偏移
fn get_pml2_index(phys: bsl::SafeU64) -> bsl::SafeU64 {
    let mask = bsl::to_u64(0x1FF);
    let shift = bsl::to_u64(21);
    (phys >> shift) & mask
}

// 返回物理地址的EPT的偏移
fn get_pml1_index(phys: bsl::SafeU64) -> bsl::SafeU64 {
    let mask = bsl::to_u64(0x1FF);
    let shift = bsl::to_u64(12);
    (phys >> shift) & mask
}
