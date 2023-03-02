use bitfield_struct::bitfield;

/// @struct EptPointerT
///
/// <!-- description -->
///   @brief Defines the layout of a extended-page-table pointer (EPTP).
#[bitfield(u64)]
pub struct EptPointerT {
    /// [Bits 2:0] EPT paging-structure memory type:
    /// - 0 = Uncacheable (UC)
    /// - 6 = Write-back (WB)
    /// Other values are reserved.
    #[bits(3)]
    pub memory_type: usize,
    /// [Bits 5:3] This value is 1 less than the EPT page-walk length
    #[bits(3)]
    pub page_walk_len: usize,
    /// [Bit 6] Setting this control to 1 enables accessed and dirty flags for EPT
    pub enable_access_and_drity: bool,
    /// defines the "ignored" field in the page
    #[bits(5)]
    ignore1: usize,
    /// [Bits 47:12] Bits N-1:12 of the physical address of the 4-KByte aligned EPT PML4 table
    #[bits(36)]
    pub pfn: usize,
    /// defines the "ignored" field in the page
    #[bits(16)]
    ignore2: usize,
}
