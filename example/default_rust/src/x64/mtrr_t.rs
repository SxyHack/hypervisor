use core::cmp::min;

const MTRR_MAX_RANGES: usize = 512;

/// @brief defines the MSR_IA32_MTRR_CAPABILITIES
const MSR_IA32_MTRR_CAPABILITIES: u32 = 0x000000FE;
/// @brief defines the MSR_IA32_MTRR_CAPABILITIES.VCNT field
const MSR_IA32_MTRR_CAP_VCNT: u64 = 0x00000000000000FF;

// /// @brief defines the MSR_IA32_MTRR_DEFTYPE MSR
// const MSR_IA32_MTRR_DEFTYPE: u32 = 0x000002FF;
// /// @brief defines the MSR_IA32_MTRR_DEFTYPE MSR type field
// const MSR_IA32_MTRR_DEFTYPE_TYPE: u64 = 0x00000000000000FF;
// /// @brief defines the MSR_IA32_MTRR_DEFTYPE MSR fixed range enable field
// const MSR_IA32_MTRR_DEFTYPE_FE: u64 = 0x0000000000000400;
// /// @brief defines the MSR_IA32_MTRR_DEFTYPE MSR enable field
// const MSR_IA32_MTRR_DEFTYPE_E: u64 = 0x0000000000000800;

const MSR_IA32_MTRR_PHYSBASE0: u32 = 0x00000200;
const MSR_IA32_MTRR_PHYSMASK0: u32 = 0x00000201;

const CPUID_LP_ADDRESS_SIZE: usize = 0x80000008;
const CPUID_LP_ADDRESS_SIZE_PHYS_ADDR_BITS: usize = 0x000000FF;

/// memory types
/// @brief defines the uncacheable memory type
pub const MEMORY_TYPE_UC: usize = 0;
// /// @brief defines the write-combine memory type
// const MEMORY_TYPE_WC: usize = 1;
// /// @brief defines the write-through memory type
// const MEMORY_TYPE_WT: usize = 4;
// /// @brief defines the write-protect memory type
// const MEMORY_TYPE_WP: usize = 5;
/// @brief defines the write-back memory type
pub const MEMORY_TYPE_WB: usize = 6;

#[derive(Debug, Clone, Copy)]
pub struct MtrrRangeT {
    addr: bsl::SafeU64,
    size: bsl::SafeU64,
    memory_type: bsl::SafeU64,
}

impl MtrrRangeT {
    pub const fn default() -> Self {
        Self {
            addr: bsl::SafeU64::new(0),
            size: bsl::SafeU64::new(0),
            memory_type: bsl::SafeU64::new(0),
        }
    }

    pub fn new(addr: bsl::SafeU64, size: bsl::SafeU64, memory_type: bsl::SafeU64) -> Self {
        Self {
            addr,
            size,
            memory_type,
        }
    }
}

/// <!-- description -->
/// Parses the MTRRs and provides a continuous, non-overlapping
/// view of the ranges as needed.
#[derive(Debug, Copy, Clone)]
pub struct MtrrT {
    pub phys_addr_bits: bsl::SafeU64,
    pub phys_addr_end: bsl::SafeU64,
    pub ranges: [MtrrRangeT; MTRR_MAX_RANGES],
    pub ranges_count: usize,
}

impl MtrrT {
    pub const fn new() -> Self {
        Self {
            phys_addr_bits: bsl::SafeU64::new(0),
            phys_addr_end: bsl::SafeU64::new(0),
            ranges: [MtrrRangeT::default(); MTRR_MAX_RANGES],
            ranges_count: 0,
        }
    }

    pub fn initialize(
        &mut self,
        sys: &syscall::BfSyscallT,
        intrinsic: &crate::IntrinsicT,
    ) -> bsl::ErrcType {
        let mut rax = bsl::to_u64(CPUID_LP_ADDRESS_SIZE);
        let mut rbx = bsl::to_u64(0);
        let mut rcx = bsl::to_u64(0);
        let mut rdx = bsl::to_u64(0);

        intrinsic.cpuid(&mut rax, &mut rbx, &mut rcx, &mut rdx);

        let pas = rax & bsl::to_u64(CPUID_LP_ADDRESS_SIZE_PHYS_ADDR_BITS);
        self.phys_addr_bits = pas;
        // let pas_bytes = bsl::SafeUMx::magic_1() << (pas.get() as usize);

        // NOTE:
        // - The next step is to get the MTRR information from the MSRs.
        //   We have to ask the kernel for this information.
        //
        let msr = bsl::to_u32(MSR_IA32_MTRR_CAPABILITIES);
        let cap = sys.bf_intrinsic_op_rdmsr(msr);
        let cap_vcnt = bsl::to_u32(cap & MSR_IA32_MTRR_CAP_VCNT);

        bsl::debug_v!(
            "cap_vcnt={} phys_addr_bits={}\n",
            cap_vcnt,
            self.phys_addr_bits
        );

        self.add_vcnt_range(cap_vcnt, pas, sys);

        return bsl::errc_success;
    }

    /// <!-- description -->
    /// 获取物理地址的内存类型
    /// 
    /// 参数表
    /// - phys: 内存物理地址
    /// 
    /// 返回内存类型(usize)
    pub fn get_memory_type(&self, phys: bsl::SafeU64) -> usize {
        let mut ret = bsl::SafeU64::new(0);

        for i in 0..self.ranges_count {
            let range = self.ranges.get(i).unwrap();
            if range.addr <= phys && phys < range.addr + range.size {
                ret = range.memory_type;
            }
        }

        ret.get() as usize
    }

    /// <!-- description -->
    ///   @brief Parses all of the variable range MTRRs and adds them
    ///     to the list.
    ///
    /// <!-- inputs/outputs -->
    ///   @param handle the handle to use
    ///   @param vcnt the total number of supported variable range MTRRs
    ///   @param pas the physical address size
    ///   @return Returns bsl::errc_success on success and bsl::errc_failure
    ///     on failure.
    ///
    fn add_vcnt_range(
        &mut self,
        vcnt: bsl::SafeU32,
        pas: bsl::SafeU64,
        sys: &syscall::BfSyscallT,
    ) -> bsl::ErrcType {
        for i in 0..vcnt.get() {
            let mtrr_phys_mask_n = bsl::to_u32(MSR_IA32_MTRR_PHYSMASK0 + (i * 2));
            let mtrr_phys_base_n = bsl::to_u32(MSR_IA32_MTRR_PHYSBASE0 + (i * 2));
            let phys_mask = sys.bf_intrinsic_op_rdmsr(mtrr_phys_mask_n);
            let phys_base = sys.bf_intrinsic_op_rdmsr(mtrr_phys_base_n);

            if !MtrrT::physmask_is_valid(phys_mask) {
                continue;
            }

            let addr = MtrrT::physbase_to_addr(phys_base);
            let size = MtrrT::physmask_to_size(phys_mask, pas);
            let memory_type = MtrrT::physbase_to_type(phys_base);

            bsl::debug!(
                "MtrrRange: [{:#016x}-{:#016x}] Type={}\n",
                addr,
                size,
                memory_type
            );

            self.phys_addr_end = self.phys_addr_end.max(addr + size);

            // if memory_type == bsl::to_u64(MEMORY_TYPE_WB) {
            //     continue;
            // }

            let ret = self.add_range(MtrrRangeT::new(addr, size, memory_type));
            if ret == bsl::errc_failure {
                bsl::error!("MTRR add range failed, {}", bsl::here());
                return ret;
            }

        }

        bsl::errc_success
    }

    /// <!-- description -->
    ///   @brief Adds a range to the list. This version of the function
    ///     does not attempt to clean up the ranges in the list. It
    ///     simply adds the range to the list and moves on.
    ///
    /// <!-- inputs/outputs -->
    ///   @param r the range to add
    ///   @return Returns bsl::errc_success on success and bsl::errc_failure
    ///     on failure.
    ///
    fn add_range(&mut self, r: MtrrRangeT) -> bsl::ErrcType {
        if let Some(item) = self.ranges.get_mut(self.ranges_count) {
            *item = r;
            self.ranges_count += 1;
            bsl::errc_success
        } else {
            bsl::error!("mtrr_t full, {}", bsl::here());
            bsl::errc_failure
        }
    }

    /// <!-- description -->
    /// @brief Returns true if the valid bit is set in physmask,false otherwise.
    /// - [Bit 11] Enables the register pair when set; disables register pair when clear.
    ///
    /// <!-- inputs/outputs -->
    ///   @param physmask the physmask to query
    ///   @return Returns true if the valid bit is set in physmask,
    ///     false otherwise.
    ///
    fn physmask_is_valid(physmask: bsl::SafeU64) -> bool {
        let mask = bsl::SafeU64::new(0b100000000000);
        let valid = physmask & mask;
        valid.is_pos()
    }

    /// <!-- description -->
    ///   @brief Returns the base address portion of physbase
    ///
    /// <!-- inputs/outputs -->
    ///   @param physbase the physbase to convert
    ///   @return Returns the base address portion of physbase
    ///
    fn physbase_to_addr(physbase: bsl::SafeU64) -> bsl::SafeU64 {
        let mask = bsl::SafeU64::new(0xFFFFFFFFFFFFF000);
        physbase & mask
    }

    /// <!-- description -->
    ///   @brief Returns the size portion of physmask using the conversion
    ///     logic defined in the manual.
    ///
    /// <!-- inputs/outputs -->
    ///   @param physmask the physmask to convert
    ///   @param pas the physical address size
    ///   @return Returns the size portion of physmask using the conversion
    ///     logic defined in the manual.
    fn physmask_to_size(physmask: bsl::SafeU64, pas: bsl::SafeU64) -> bsl::SafeU64 {
        let mask = bsl::SafeU64::new(0xFFFFFFFFFFFFF000);
        let one = bsl::SafeU64::magic_1();
        return (!(physmask & mask) & ((one << pas) - one)) + one;
    }

    fn physbase_to_type(physbase: bsl::SafeU64) -> bsl::SafeU64 {
        let mask = bsl::SafeU64::new(0x00000000000000FF);
        return physbase & mask;
    }
}
