pub mod ept_entry;
pub mod ept_pointer;

use ept_entry::*;
use ept_pointer::EptPointerT;

const EPT_PML_SIZE: usize = 512;

/// EPT
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExtentPageTable {
    // stores a pointer to the PML4
    pub pml4: *mut EptEntryT,
    // stores the physical address of the PML4
    pub pml4_phys: bsl::SafeU64,
    // stores a pointer to the PML3
    pub pml3: *mut EptEntryT,
    // stores the physical address of the PML3
    pub pml3_phys: bsl::SafeU64,
    // stores a pointer to the PML2
    pub pml2: *mut EptEntryT,
    // stores the physical address of the PML2
    pub pml2_phys: bsl::SafeU64,
    // stores true if initialize() has been executed
    pub initialized: bool,
    // stores a pointer to the EptPointerT
    eptp: EptPointerT,
}

impl ExtentPageTable {
    pub const fn new() -> Self {
        Self {
            pml4: core::ptr::null_mut(),
            pml4_phys: bsl::SafeU64::new(0),
            pml3: core::ptr::null_mut(),
            pml3_phys: bsl::SafeU64::new(0),
            pml2: core::ptr::null_mut(),
            pml2_phys: bsl::SafeU64::new(0),
            eptp: EptPointerT::new(),
            initialized: false,
        }
    }

    pub fn initialize(&mut self, sys: &syscall::BfSyscallT) -> bsl::ErrcType {
        bsl::debug_v!("initialize EPT...\n");

        self.pml4 = sys.bf_mem_op_alloc_page(&mut self.pml4_phys);
        if core::ptr::null_mut() == self.pml4 {
            bsl::error!("{}", bsl::here());
            return bsl::errc_failure;
        }

        bsl::debug!(
            "PML4.virt={:#018x} phys={:#018x}\n",
            self.pml4.addr(),
            self.pml4_phys
        );

        self.pml3 = sys.bf_mem_op_alloc_page(&mut self.pml3_phys);
        if core::ptr::null_mut() == self.pml3 {
            bsl::error!("{}", bsl::here());
            return bsl::errc_failure;
        }

        let pml3_pfn = (self.pml3_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;

        bsl::debug!(
            "PML3.virt={:#018x} phys={:#018x} pfn={:#018x}\n",
            self.pml3.addr(),
            self.pml3_phys,
            pml3_pfn
        );

        unsafe {
            (*self.pml4).set_r(true);
            (*self.pml4).set_w(true);
            (*self.pml4).set_e(true);
            (*self.pml4).set_phys(pml3_pfn);
        }

        bsl::errc_success
    }

    pub fn alloc_2m_page(
        &mut self,
        sys: &syscall::BfSyscallT,
        phys: bsl::SafeU64,
        memory_type: usize,
    ) -> bsl::ErrcType {
        // bsl::discard(sys);
        // bsl::discard(memory_type);

        let pml4_index = get_pml4_index(phys);
        if !pml4_index.is_zero() {
            bsl::error!("pml4 index is invalid, {}", bsl::here());
            return bsl::errc_failure;
        }

        let pml3_index = get_pml3_index(phys);
        // let pml3_entry = unsafe { self.pml3.add(pml3_index.get() as usize) };

        let pml2_index = get_pml2_index(phys);

        // bsl::debug!(
        //     "alloc_page_2m({}): pml4={} pml3={} pml2={}\n",
        //     memory_type,
        //     pml4_index,
        //     pml3_index,
        //     pml2_index
        // );

        bsl::errc_success
    }

    // pub fn initialize(&mut self, mtrr: &MtrrT, sys: &syscall::BfSyscallT) -> bsl::ErrcType {
    // bsl::debug_v!("initialize EPT...\n");

    // let size_of_entry = core::mem::size_of::<u64>();
    // let size_of_pml34 = size_of_entry * EPT_PML_SIZE * 2; // 8192 = 2 page
    // let size_of_pml02 = size_of_entry * EPT_PML_SIZE * EPT_PML_SIZE; // 512 page
    // let size_of_total = size_of_pml34 + size_of_pml02; // 2105344
    // let size_of_huge = 0x10000usize;
    // let cunt_of_pml2 = size_of_pml02 / size_of_huge;
    // let cunt_of_entry = size_of_total / size_of_entry;

    // bsl::print_v!(
    //     "huge block detail: total={} page={} count={}\n",
    //     size_of_total,
    //     size_of_huge,
    //     cunt_of_entry
    // );

    // // 分配 PML4 & PML3 的物理内存
    // self.pml4 =
    //     sys.bf_mem_op_alloc_huge::<EptEntryT>(bsl::to_u64(size_of_pml34), &mut self.pml4_phys);

    // // 分配 PML2 的物理内存, HugePool每次最多能分配1M, 必须4K对齐
    // for i in 0..cunt_of_pml2 {
    //     let mut phys = bsl::SafeU64::new(0);
    //     let virt = sys.bf_mem_op_alloc_huge::<EptEntryT>(bsl::to_u64(size_of_huge), &mut phys);
    //     if virt == core::ptr::null_mut() {
    //         bsl::print_v!("{}", bsl::here());
    //         return bsl::errc_failure;
    //     }

    //     bsl::print_v!(
    //         "{:#02X}: PHYS={:#016X} VIRT={:#016X} PAGE={:#016X}\n",
    //         i,
    //         phys,
    //         virt.addr(),
    //         size_of_huge
    //     );
    // }

    // // dump huge pool
    // syscall::bf_debug_op_dump_huge_pool();

    // self.pml3_phys = self.pml4_phys + syscall::HYPERVISOR_PAGE_SIZE;
    // self.pml3 = unsafe { self.pml4.add(EPT_PML_SIZE) };

    // self.pml2_phys = self.pml3_phys + syscall::HYPERVISOR_PAGE_SIZE;
    // self.pml2 = unsafe { self.pml3.add(EPT_PML_SIZE) };

    // // 相当于 pml3_phys / syscall::HYPERVISOR_PAGE_SIZE;
    // let pml3_pfn = (self.pml3_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
    // unsafe {
    //     (*self.pml4).set_phys(pml3_pfn);
    //     (*self.pml4).set_w(true);
    //     (*self.pml4).set_r(true);
    //     (*self.pml4).set_e(true);
    // }

    // let pml3_virt = sys.bf_vm_op_map_direct::<u8>(syscall::BF_ROOT_VMID, self.pml3_phys);

    // bsl::print_v!(
    //     "Allocated PML4 & PML3\n{:#016X}\n{:#016X} - {:#016X}\n",
    //     self.pml4_phys,
    //     self.pml3_phys, self.pml3_phys
    // );
    // bsl::print_v!("PML4.pml3_pfn={:#08X}\n", pml3_pfn);

    // // unsafe {
    // //     core::intrinsics::breakpoint();
    // // };

    // // 初始化 PML3 入口
    // for i in 0..EPT_PML_SIZE {
    //     let mut entry = unsafe { *self.pml3.add(i) };
    //     entry.set_w(true);
    //     entry.set_r(true);
    //     entry.set_e(true);
    //     let pml3_indx = bsl::to_u64(i);
    //     let pml2_phys = self.pml2_phys + (pml3_indx * syscall::HYPERVISOR_PAGE_SIZE);
    //     entry.set_phys((pml2_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize);
    // }

    // let pml2_count = EPT_PML_SIZE * EPT_PML_SIZE;
    // // 初始化 PML2 入口
    // for i in 0..pml2_count {
    //     let mut entry = unsafe { *self.pml2.add(i) };

    //     entry.set_w(true);
    //     entry.set_r(true);
    //     entry.set_e(true);
    //     entry.set_large_page(true);
    //     entry.set_phys(i << 9);

    //     if i == 0 {
    //         entry.set_mem_type(mtrr_t::MEMORY_TYPE_UC);
    //         continue;
    //     }

    //     entry.set_mem_type(mtrr_t::MEMORY_TYPE_WB);
    // }

    // let pml4_pfn = (self.pml4_phys >> syscall::HYPERVISOR_PAGE_SHIFT).get() as usize;
    // self.eptp.set_memory_type(mtrr_t::MEMORY_TYPE_WB);
    // self.eptp.set_page_walk_len(3);
    // self.eptp.set_enable_access_and_drity(false);
    // self.eptp.set_pfn(pml4_pfn);

    // let mask: u64 = self.eptp.into();
    // bsl::debug_v!("installed EPT, EPTP: {:#08X}\n", mask);

    // self.initialized = true;
    //     bsl::errc_success
    // }

    pub fn eptp(&self) -> bsl::SafeU64 {
        let mask: u64 = self.eptp.into();
        bsl::to_u64(mask)
    }
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
