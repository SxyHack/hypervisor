use bitfield_struct::bitfield;

/// @struct EptEntryPdt
///
/// <!-- description -->
///   @brief Defines the layout of a nested page-directory table entry
///     (EPDTE).
///
#[bitfield(u64)]
struct EptEntryPdt {
    /// @brief defines the "read access" field in the page
    r: bool,
    /// @brief defines the "write access" field in the page
    w: bool,
    /// @brief defines the "execute access" field in the page
    e: bool,
    /// @brief defines the "memory type" field in the page
    #[bits(3)]
    mem_type: usize,
    /// @brief defines the "ignore pat" field in the page
    ignore_pat: bool,
    /// @brief defines the "large page" field in the page
    large_page: bool,
    /// @brief defines the "accessed" field in the page
    a: bool,
    /// @brief defines the "dirty" field in the page
    d: bool,
    /// @brief defines the "user execute access" field in the page
    e_user: bool,
    /// @brief defines the "ignored" field in the page
    ignored1: bool,
    /// @brief defines the "physical address" field in the page
    #[bits(40)]
    phys: usize,
    /// @brief defines the "ignored" field in the page
    #[bits(11)]
    ignored2: usize,
    /// @brief defines the "virtualization exception" field in the page
    ve: bool
}