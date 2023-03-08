const MTRR_MAX_RANGES: usize = 512;

/// defines the MSR_IA32_MTRR_CAPABILITIES
const MSR_IA32_MTRR_CAPABILITIES: u32 = 0x000000FE;
/// defines the MSR_IA32_MTRR_CAPABILITIES.VCNT field
const MSR_IA32_MTRR_CAP_VCNT: u64 = 0x00000000000000FF;

const MSR_IA32_MTRR_PHYSBASE0: u32 = 0x00000200;
const MSR_IA32_MTRR_PHYSMASK0: u32 = 0x00000201;

// const CPUID_LP_ADDRESS_SIZE: u32 = 0x80000008;
// const CPUID_LP_ADDRESS_SIZE_PHYS_ADDR_BITS: u32 = 0x000000FF;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryType {
    MemoryTypeUC = 0,
    MemoryTypeWB = 6,
}

impl From<u8> for MemoryType {
    fn from(value: u8) -> MemoryType {
        match value {
            0 => MemoryType::MemoryTypeUC,
            6 => MemoryType::MemoryTypeWB,
            _ => panic!("Unknown value:{}", value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MtrrRange {
    pub addr: u64,
    pub size: u32,
    pub kind: MemoryType,
}

impl MtrrRange {
    pub const fn default() -> Self {
        Self {
            addr: 0,
            size: 0,
            kind: MemoryType::MemoryTypeUC,
        }
    }

    pub fn new(addr: u64, size: u32, kind: MemoryType) -> Self {
        Self { addr, size, kind }
    }

    pub fn end(&self) -> u64 {
        self.addr + self.size as u64
    }
}

/// ranges
#[derive(Debug, Clone, Copy)]
pub struct MtrrT {
    ranges: [MtrrRange; MTRR_MAX_RANGES],
    ranges_cnt: u32,
    // 存储最大的物理地址
    pub max_phys: u64,
}

impl MtrrT {
    pub const fn new() -> Self {
        Self {
            ranges: [MtrrRange::default(); MTRR_MAX_RANGES],
            ranges_cnt: 0,
            max_phys: 0,
        }
    }

    pub fn initialize(
        &mut self,
        sys: &syscall::BfSyscallT,
        intrinsic: &crate::IntrinsicT,
    ) -> bsl::ErrcType {
        // 目前只构建 VCNT
        self.build_vcnt_ranges(sys, intrinsic);
        bsl::errc_success
    }

    fn build_vcnt_ranges(
        &mut self,
        sys: &syscall::BfSyscallT,
        intrinsic: &crate::IntrinsicT,
    ) -> bsl::ErrcType {
        let phys_bits = intrinsic.phys_bits();
        bsl::debug!("{}", bsl::here());
        let msr = bsl::to_u32(MSR_IA32_MTRR_CAPABILITIES);
        bsl::debug!("{}", bsl::here());
        let cap = sys.bf_intrinsic_op_rdmsr(msr);
        bsl::debug!("{}", bsl::here());
        let cap_vcnt = bsl::to_u32(cap & MSR_IA32_MTRR_CAP_VCNT);
        bsl::debug!("{}", bsl::here());
        let mut max_phys = bsl::to_u64(0);
        bsl::debug_v!("cap_vcnt={} phys_bits={}\n", cap_vcnt, phys_bits);

        for i in 0..cap_vcnt.get() {
            let mtrr_phys_mask_n = bsl::to_u32(MSR_IA32_MTRR_PHYSMASK0 + (i * 2));
            let mtrr_phys_base_n = bsl::to_u32(MSR_IA32_MTRR_PHYSBASE0 + (i * 2));
            let phys_mask = sys.bf_intrinsic_op_rdmsr(mtrr_phys_mask_n);
            let phys_base = sys.bf_intrinsic_op_rdmsr(mtrr_phys_base_n);

            if !physmask_is_valid(phys_mask) {
                continue;
            }

            let addr = physbase_to_addr(phys_base);
            let size = physmask_to_size(phys_mask, bsl::to_u64(phys_bits));
            let kind = physbase_to_type(phys_base);
            let kind = MemoryType::from(kind.get() as u8);

            bsl::debug!(
                "MtrrRange: [{:#016x}-{:#016x}] Type={:?}\n",
                addr,
                size,
                kind
            );

            max_phys = max_phys.max(addr + size);

            let ret = self.add_range(MtrrRange::new(addr.get(), size.get() as u32, kind));
            if !ret.success() {
                bsl::error!("add_range failed {}", bsl::here());
                return bsl::errc_failure;
            }
        }

        self.max_phys = max_phys.get();
        bsl::errc_success
    }

    /// Adds a range to the list. This version of the function
    /// does not attempt to clean up the ranges in the list. It
    /// simply adds the range to the list and moves on.
    ///
    /// <!-- inputs/outputs -->
    ///   @param r the range to add
    ///   @return Returns bsl::errc_success on success and bsl::errc_failure
    ///     on failure.
    ///
    fn add_range(&mut self, r: MtrrRange) -> bsl::ErrcType {
        let cnt = self.ranges_cnt as usize;

        if cnt >= self.ranges.len() {
            bsl::error!("MTRR ranges is full, {}", bsl::here());
            return bsl::errc_failure;
        }

        self.ranges[cnt] = r;
        self.ranges_cnt += 1;

        bsl::errc_success
    }

    /// 获取物理地址的内存类型
    ///
    /// 参数表
    /// - phys: 内存物理地址
    ///
    /// 返回内存类型(usize)
    pub fn get_memory_type(&self, phys: bsl::SafeU64) -> u8 {
        let phys = phys.get();
        let mut ret = 0;

        for i in 0..self.ranges_cnt {
            let range = self.ranges[i as usize];
            if range.addr <= phys && phys < range.end() {
                ret = range.kind as u8;
            }
        }

        ret
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
