use bitfield_struct::bitfield;

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
    /// @brief defines the "memory type" field in the page
    #[bits(3)]
    pub mem_type: usize,
    /// @brief defines the "ignore pat" field in the page
    pub ignore_pat: bool,
    /// @brief defines the "large page" field in the page
    pub large_page: bool,
    /// @brief defines the "accessed" field in the page
    pub a: bool,
    /// @brief defines the "dirty" field in the page
    pub d: bool,
    /// @brief defines the "user execute access" field in the page
    pub e_user: bool,
    /// @brief defines the "ignored" field in the page
    pub ignored1: bool,
    /// @brief defines the "physical address" field in the page
    #[bits(40)]
    pub phys: usize,
    /// @brief defines the "ignored" field in the page
    #[bits(11)]
    pub ignored2: usize,
    /// @brief defines the "virtualization exception" field in the page
    pub ve: bool
}