use bitfield_struct::bitfield;

/// @struct EptEntryPml4
///
/// <!-- description -->
///   @brief Defines the layout of a nested page-map level-4 table entry
///     (EPML4TE).
///
#[bitfield(u64)]
struct EptEntryPml4 {
    /// @brief defines the "read access" field in the page
    r: bool,
    /// @brief defines the "write access" field in the page
    w: bool,
    /// @brief defines the "execute access" field in the page
    e: bool,
    /// @brief defines the "must be zero" field in the page
    #[bits(5)]
    mbz: usize,
    /// @brief defines the "accessed" field in the page
    a: bool,
    /// @brief defines the "ignored" field in the page
    ignored1: bool,
    /// @brief defines the "user execute access" field in the page
    e_user: bool,
    /// @brief defines the "ignored" field in the page
    ignored2: bool,
    /// @brief defines the "physical address" field in the page
    #[bits(40)]
    phys: usize,
    /// @brief defines the "ignored" field in the page
    #[bits(12)]
    ignored3: usize,
}