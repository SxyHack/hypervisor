use bitfield_struct::bitfield;

// const EPT_ENTRY_COUNT:usize = 512;

/// @struct EptEntryT
///
/// <!-- description -->
///   @brief Defines the layout of a nested page-directory table entry
///     (EPDTE).
///
#[bitfield(u64)]
pub struct EptEntryT {
    /// @brief defines the "read access" field in the page
    pub r: bool,
    /// @brief defines the "write access" field in the page
    pub w: bool,
    /// @brief defines the "execute access" field in the page
    pub e: bool,
    /// @brief defines the "reserved(must be 0)" field in the page
    #[bits(5)]
    pub mbz: usize,
    /// @brief defines the "accessed" field in the page
    pub a: bool,
    /// @brief defines the "ignored" field in the page
    pub ignored1: bool,
    /// @brief defines the "user execute access" field in the page
    pub e_user: bool,
    /// @brief defines the "ignored" field in the page
    pub ignored2: bool,
    /// @brief defines the "physical address" field in the page
    #[bits(40)]
    pub phys: usize,
    /// @brief defines the "ignored" field in the page
    #[bits(12)]
    pub ignored3: usize,
}
