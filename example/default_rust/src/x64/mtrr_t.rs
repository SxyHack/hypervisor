use alloc::vec;
use alloc::vec::Vec;

const MTRR_MAX_RANGES: usize = 512;

/// @brief defines the MSR_IA32_MTRR_CAPABILITIES
const MSR_IA32_MTRR_CAPABILITIES: bsl::SafeU32 = bsl::to_u32(0x000000FE);
/// @brief defines the MSR_IA32_MTRR_CAPABILITIES.VCNT field
const MSR_IA32_MTRR_CAP_VCNT: bsl::SafeUMx = bsl::to_umx(0x00000000000000FF);

/// @brief defines the MSR_IA32_MTRR_DEFTYPE MSR
const MSR_IA32_MTRR_DEFTYPE: bsl::SafeU32 = bsl::to_u32(0x000002FF);

const MSR_IA32_MTRR_PHYSBASE0: usize = 0x00000200;
const MSR_IA32_MTRR_PHYSMASK0: usize = 0x00000201;

const CPUID_LP_ADDRESS_SIZE: usize = 0x80000008;
const CPUID_LP_ADDRESS_SIZE_PHYS_ADDR_BITS: usize = 0x000000FF;


struct MtrrRangeT {
    addr: bsl::SafeUMx,
    size: bsl::SafeUMx,
    memory_type: bsl::SafeUMx,
}

/// <!-- description -->
/// Parses the MTRRs and provides a continuous, non-overlapping
/// view of the ranges as needed.
struct MtrrT {
    ranges: Vec<MtrrRangeT>,
}

impl MtrrT {
    pub fn new() -> Self {
        Self {
            ranges: vec![],
        }
    }

    pub fn build(sys: &syscall::BfSyscallT, intrinsic: &crate::IntrinsicT) -> bsl::ErrcType {
        let mut rax = bsl::to_u64(CPUID_LP_ADDRESS_SIZE);
        let mut rbx = bsl::to_u64(0);
        let mut rcx = bsl::to_u64(0);
        let mut rdx = bsl::to_u64(0);

        intrinsic.cpuid(&mut rax, &mut rbx, &mut rcx, &mut rdx);

        let pas = rax & bsl::to_u64(CPUID_LP_ADDRESS_SIZE_PHYS_ADDR_BITS);
        let pas_bytes = bsl::SafeU64::magic_1() << pas;
        bsl::print_v!("pas={}, bytes={}", pas, pas_bytes);

        /// NOTE:
        /// - The next step is to get the MTRR information from the MSRs.
        ///   We have to ask the kernel for this information.
        ///
        let cap = sys.bf_intrinsic_op_rdmsr(MSR_IA32_MTRR_CAPABILITIES);
        if !cap {
            bsl::print_v!("{}", bsl::here());
            return bsl::errc_failure;
        }

        let def_type = sys.bf_intrinsic_op_rdmsr(MSR_IA32_MTRR_DEFTYPE);
        if !def_type {
            bsl::print_v!("{}", bsl::here());
            return bsl::errc_failure;
        }

        let cap_vcnt = bsl::to_u32(cap & MSR_IA32_MTRR_CAP_VCNT);

        return bsl::errc_success;
    }

    /// static
    pub fn physmask_to_size(mask: u32, pas: usize) -> u32 {
        0
    }
}