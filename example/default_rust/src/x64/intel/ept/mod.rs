pub mod ept_entry;
pub mod ept_pml4;

// #[path = "constants.rs"]
// #[doc(hidden)]
// pub mod constants;
// pub use constants::*;

use ept_entry::EptEntryT;

const EPT_PML_SIZE: usize = 512;

/// EPT
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExtentPageTable {
    pml4: *mut EptEntryT,
    pml4_phys: bsl::SafeU64,

    pml3: *mut EptEntryT,
    pml3_phys: bsl::SafeU64,

    pml2: *mut EptEntryT,
    /// defined pml2 physical base address
    pml2_phys: bsl::SafeU64,
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
        }
    }

    pub fn initialize(&mut self, sys: &syscall::BfSyscallT) -> bsl::ErrcType {
        bsl::debug_v!("initialize EPT...\n");
        let size_of_entry = core::mem::size_of::<EptEntryT>();

        let size = bsl::to_u64(EPT_PML_SIZE * EPT_PML_SIZE);
        let template = EptEntryT::new()
            .with_r(true)
            .with_w(true)
            .with_e(true)
            .with_large_page(true);
        self.pml2 = sys.bf_mem_op_alloc_huge(size, &mut self.pml2_phys);

        // 相当于C++中的 __stosq 函数
        for i in 0..size.get() as usize {
            unsafe {
                let entry = self.pml2.add(i);
                *entry = template;
            }
        }

        self.pml3 = sys.bf_mem_op_alloc_page(&mut self.pml3_phys);
        if self.pml3 == core::ptr::null_mut() {
            bsl::print_v!("{}", bsl::here());
            return bsl::errc_failure;
        }

        let template = EptEntryT::new().with_r(true).with_w(true).with_e(true);
        // 相当于C++中的 __stosq 函数
        for i in 0..512usize {
            unsafe {
                let entry = self.pml3.add(i);
                *entry = template;
            }
        }

        self.pml4 = sys.bf_mem_op_alloc_page(&mut self.pml4_phys);
        if self.pml4 == core::ptr::null_mut() {
            bsl::print_v!("{}", bsl::here());
            return bsl::errc_failure;
        }

        bsl::print_v!("PML3 PhysAddr: {:#06x}\n", self.pml3_phys);
        dump_pml_hex(self.pml3, size_of_entry * EPT_PML_SIZE);

        unsafe {
            (*self.pml4).set_r(true);
            (*self.pml4).set_w(true);
            (*self.pml4).set_e(true);
            (*self.pml4).set_phys(self.pml3_phys.get() as usize);
        }

        bsl::errc_success
    }

    unsafe fn dump_pml3(&self) {
        bsl::print_v!("dump ept pml3: {:#032X}\n", self.pml3_phys);

        for i in 0..EPT_PML_SIZE {
            let p = *self.pml3.add(i);
            let v: u64 = p.into();
            // bsl::print_v!("{}{}{}", p.r() as u8, p.w() as u8, p.e() as u8);
            bsl::print_v!("{:#06x}", v);

            if i < EPT_PML_SIZE - 1 {
                bsl::print_v!(" ");
            }

            if i > 0 && i % 10 == 0 {
                bsl::print_v!("\n");
            }
        }

        bsl::print_v!("\n");
    }
}

fn dump_pml_hex(entry: *mut EptEntryT, size: usize) {
    bsl::print_v!("dump ept pml:\n");

    for i in 0..size {
        let raw = entry as *mut u8;

        unsafe {
            bsl::print_v!("{:#02x}", *raw);
        }

        if i < size - 1 {
            bsl::print_v!(" ");
        }

        if i + 1 % 32 == 0 {
            bsl::print_v!("\n");
        }
    }

    bsl::print_v!("\n");
}
