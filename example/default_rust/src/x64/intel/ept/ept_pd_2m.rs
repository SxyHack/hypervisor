use bitfield_struct::bitfield;


#[bitfield(u64)]
pub struct EptMapPd {
    /// @brief defines the "read access" field in the page
    pub r: bool,
    /// @brief defines the "write access" field in the page
    pub w: bool,
    /// @brief defines the "execute access" field in the page
    pub e: bool,
    /// [bit 5:3] memory type for this 2-MByte page (see Section 27.2.7)
    #[bits(3)]
    pub memory_type: u8,
    /// [bit 6] Ignore PAT memory type for this 2-MByte page
    pub ignore_pat: bool,
    /// [bit 7] Must be 1 (otherwise, this entry references an EPT page table)
    pub is_map: bool,
    /// [bit 8] If bit 6 of EPTP is 1, accessed flag for EPT
    pub a: bool,
    /// [bit 9] If bit 6 of EPTP is 1, dirty flag for EPT
    pub d: bool,
    /// [bit 10] defines the "user execute access" field in the page
    pub e_user: bool,
    /// @brief defines the "ignored" field in the page
    ignored1: bool,
    /// [bits 20:12] Reserved (must be 0)
    #[bits(9)]
    reserved: u16,
    /// [bits 51:21] Physical address of the 2-MByte page referenced by this entry
    #[bits(31)]
    pub phys: usize,
    /// @brief defines the "ignored" field in the page
    #[bits(12)]
    ignored2: usize,
}