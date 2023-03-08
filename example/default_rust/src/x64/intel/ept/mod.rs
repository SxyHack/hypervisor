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
    //     "PML4({}) -> PML3({}) -> PML2({})\n",
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

    bsl::debug!(
        "alloc pdt-{}({:#06x}), virt={:#010x}, phys={:#010x} hex={:?}\n",
        pdpt_index,
        size,
        pdpt.tables[pdpt_index],
        pdt_phys,
        pdpt.entries[pdpt_index],
    );

    pdt_virt.addr()
}

pub fn eptp(gs: &mut crate::GsT) -> usize {
    let pml4_pfn = (gs.ept_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
    let ret = EptPointerT::new()
        .with_memory_type(6)
        .with_enable_access_and_drity(false)
        .with_page_walk_len(3)
        .with_pfn(pml4_pfn);

    let val:u64 = ret.into();
    val as usize
}

// EPT
// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// pub struct ExtentPageTable {
//     // stores a pointer to the PML4
//     pub pml4: EptEntryPml4,
//     // stores the physical address of the PML4
//     pub pml4_phys: bsl::SafeU64,
//     // stores a pointer to the PML3
//     // pub pml3: *mut EptEntryT,
//     // stores the physical address of the PML3
//     // pub pml3_phys: bsl::SafeU64,
//     // stores a pointer to the PML2
//     // pub pml2: *mut EptEntryT,
//     // // stores the physical address of the PML2
//     // pub pml2_phys: bsl::SafeU64,
//     // stores true if initialize() has been executed
//     // pub initialized: bool,
//     // stores a pointer to the EptPointerT
//     // eptp: EptPointerT,
// }

// impl ExtentPageTable {
//     pub const fn new() -> Self {
//         Self {
//             pml4: core::ptr::null_mut(),
//             pml4_phys: bsl::SafeU64::new(0),
//             // pml3: core::ptr::null_mut(),
//             // pml3_phys: bsl::SafeU64::new(0),
//             // pml2: core::ptr::null_mut(),
//             // pml2_phys: bsl::SafeU64::new(0),
//             eptp: EptPointerT::new(),
//             initialized: false,
//         }
//     }

//     pub fn initialize(&mut self, sys: &syscall::BfSyscallT) -> bsl::ErrcType {
//         bsl::debug_v!("initialize EPT...\n");

//         self.pml4 = sys.bf_mem_op_alloc_page(&mut self.pml4_phys);
//         if core::ptr::null_mut() == self.pml4 {
//             bsl::error!("{}", bsl::here());
//             return bsl::errc_failure;
//         }

//         bsl::debug!(
//             "PML4.virt={:#018x} phys={:#018x}\n",
//             self.pml4.addr(),
//             self.pml4_phys
//         );

//         // self.pml3 = sys.bf_mem_op_alloc_page(&mut self.pml3_phys);
//         // if core::ptr::null_mut() == self.pml3 {
//         //     bsl::error!("{}", bsl::here());
//         //     return bsl::errc_failure;
//         // }

//         // let pml3_pfn = (self.pml3_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;

//         // bsl::debug!(
//         //     "PML3.virt={:#018x} phys={:#018x} pfn={:#018x}\n",
//         //     self.pml3.addr(),
//         //     self.pml3_phys,
//         //     pml3_pfn
//         // );

//         // unsafe {
//         //     (*self.pml4).set_r(true);
//         //     (*self.pml4).set_w(true);
//         //     (*self.pml4).set_e(true);
//         //     (*self.pml4).set_phys(pml3_pfn);
//         // }

//         bsl::errc_success
//     }

//     pub fn alloc_2m_page(
//         &mut self,
//         sys: &syscall::BfSyscallT,
//         phys: bsl::SafeU64,
//         memory_type: usize,
//     ) -> bsl::ErrcType {
//         // bsl::discard(sys);
//         // bsl::discard(memory_type);

//         let pml4_index = get_pml4_index(phys);
//         if !pml4_index.is_zero() {
//             bsl::error!("pml4 index is invalid, {}", bsl::here());
//             return bsl::errc_failure;
//         }

//         let pml3_index = get_pml3_index(phys);
//         // let pml3_entry = unsafe { self.pml3.add(pml3_index.get() as usize) };
//         let pml2_index = get_pml2_index(phys);

//         bsl::debug!(
//             "alloc_page_2m({}): pml4={} pml3={} pml2={}\n",
//             memory_type,
//             pml4_index,
//             pml3_index,
//             pml2_index
//         );

//         bsl::errc_success
//     }

//     // pub fn initialize(&mut self, mtrr: &MtrrT, sys: &syscall::BfSyscallT) -> bsl::ErrcType {
//     // bsl::debug_v!("initialize EPT...\n");

//     // let size_of_entry = core::mem::size_of::<u64>();
//     // let size_of_pml34 = size_of_entry * EPT_PML_SIZE * 2; // 8192 = 2 page
//     // let size_of_pml02 = size_of_entry * EPT_PML_SIZE * EPT_PML_SIZE; // 512 page
//     // let size_of_total = size_of_pml34 + size_of_pml02; // 2105344
//     // let size_of_huge = 0x10000usize;
//     // let cunt_of_pml2 = size_of_pml02 / size_of_huge;
//     // let cunt_of_entry = size_of_total / size_of_entry;

//     // bsl::print_v!(
//     //     "huge block detail: total={} page={} count={}\n",
//     //     size_of_total,
//     //     size_of_huge,
//     //     cunt_of_entry
//     // );

//     // // 分配 PML4 & PML3 的物理内存
//     // self.pml4 =
//     //     sys.bf_mem_op_alloc_huge::<EptEntryT>(bsl::to_u64(size_of_pml34), &mut self.pml4_phys);

//     // // 分配 PML2 的物理内存, HugePool每次最多能分配1M, 必须4K对齐
//     // for i in 0..cunt_of_pml2 {
//     //     let mut phys = bsl::SafeU64::new(0);
//     //     let virt = sys.bf_mem_op_alloc_huge::<EptEntryT>(bsl::to_u64(size_of_huge), &mut phys);
//     //     if virt == core::ptr::null_mut() {
//     //         bsl::print_v!("{}", bsl::here());
//     //         return bsl::errc_failure;
//     //     }

//     //     bsl::print_v!(
//     //         "{:#02X}: PHYS={:#016X} VIRT={:#016X} PAGE={:#016X}\n",
//     //         i,
//     //         phys,
//     //         virt.addr(),
//     //         size_of_huge
//     //     );
//     // }

//     // // dump huge pool
//     // syscall::bf_debug_op_dump_huge_pool();

//     // self.pml3_phys = self.pml4_phys + syscall::HYPERVISOR_PAGE_SIZE;
//     // self.pml3 = unsafe { self.pml4.add(EPT_PML_SIZE) };

//     // self.pml2_phys = self.pml3_phys + syscall::HYPERVISOR_PAGE_SIZE;
//     // self.pml2 = unsafe { self.pml3.add(EPT_PML_SIZE) };

//     // // 相当于 pml3_phys / syscall::HYPERVISOR_PAGE_SIZE;
//     // let pml3_pfn = (self.pml3_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
//     // unsafe {
//     //     (*self.pml4).set_phys(pml3_pfn);
//     //     (*self.pml4).set_w(true);
//     //     (*self.pml4).set_r(true);
//     //     (*self.pml4).set_e(true);
//     // }

//     // let pml3_virt = sys.bf_vm_op_map_direct::<u8>(syscall::BF_ROOT_VMID, self.pml3_phys);

//     // bsl::print_v!(
//     //     "Allocated PML4 & PML3\n{:#016X}\n{:#016X} - {:#016X}\n",
//     //     self.pml4_phys,
//     //     self.pml3_phys, self.pml3_phys
//     // );
//     // bsl::print_v!("PML4.pml3_pfn={:#08X}\n", pml3_pfn);

//     // // unsafe {
//     // //     core::intrinsics::breakpoint();
//     // // };

//     // // 初始化 PML3 入口
//     // for i in 0..EPT_PML_SIZE {
//     //     let mut entry = unsafe { *self.pml3.add(i) };
//     //     entry.set_w(true);
//     //     entry.set_r(true);
//     //     entry.set_e(true);
//     //     let pml3_indx = bsl::to_u64(i);
//     //     let pml2_phys = self.pml2_phys + (pml3_indx * syscall::HYPERVISOR_PAGE_SIZE);
//     //     entry.set_phys((pml2_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize);
//     // }

//     // let pml2_count = EPT_PML_SIZE * EPT_PML_SIZE;
//     // // 初始化 PML2 入口
//     // for i in 0..pml2_count {
//     //     let mut entry = unsafe { *self.pml2.add(i) };

//     //     entry.set_w(true);
//     //     entry.set_r(true);
//     //     entry.set_e(true);
//     //     entry.set_large_page(true);
//     //     entry.set_phys(i << 9);

//     //     if i == 0 {
//     //         entry.set_mem_type(mtrr_t::MEMORY_TYPE_UC);
//     //         continue;
//     //     }

//     //     entry.set_mem_type(mtrr_t::MEMORY_TYPE_WB);
//     // }

//     // let pml4_pfn = (self.pml4_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
//     // self.eptp.set_memory_type(mtrr_t::MEMORY_TYPE_WB);
//     // self.eptp.set_page_walk_len(3);
//     // self.eptp.set_enable_access_and_drity(false);
//     // self.eptp.set_pfn(pml4_pfn);

//     // let mask: u64 = self.eptp.into();
//     // bsl::debug_v!("installed EPT, EPTP: {:#08X}\n", mask);

//     // self.initialized = true;
//     //     bsl::errc_success
//     // }

//     pub fn eptp(&self) -> bsl::SafeU64 {
//         let mask: u64 = self.eptp.into();
//         bsl::to_u64(mask)
//     }
// }

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
