use bitfield_struct::bitfield;

pub const EPT_ENTRY_COUNT: usize = 512;

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
    mbz: usize,
    /// @brief defines the "accessed" field in the page
    pub a: bool,
    /// @brief defines the "ignored" field in the page
    ignored1: bool,
    /// @brief defines the "user execute access" field in the page
    pub e_user: bool,
    /// @brief defines the "ignored" field in the page
    ignored2: bool,
    /// @brief defines the "physical address" field in the page
    #[bits(40)]
    pub phys: usize,
    /// @brief defines the "ignored" field in the page
    #[bits(12)]
    ignored3: usize,
}

/// PML4
/// Defines the layout of a page-map level-4 table (pml4)
///
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EptEntryPml4 {
    /// stores the entries for this page table
    pub entries: [EptEntryT; EPT_ENTRY_COUNT],
    /// stores pointers to child tables
    pub tables: [usize; EPT_ENTRY_COUNT],
}

/// PML3
/// Defines the layout of a page-directory-pionter table (pdpt).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EptEntryPdpt {
    /// stores the entries for this page table
    pub entries: [EptEntryT; EPT_ENTRY_COUNT],
    /// stores pointers to child tables
    pub tables: [usize; EPT_ENTRY_COUNT],
}

/// PML2
/// Defines the layout of a page-directory table (pdt).
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EptEntryPdt {
    /// stores the entries for this page table
    pub entries: [u64; EPT_ENTRY_COUNT],
    /// stores pointers to child(PML1) tables
    pub tables: [u64; EPT_ENTRY_COUNT],
}

/// PML1
/// defines total number of entries in the PT
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EptEntryPt {
    /// stores the entries for this page table
    pub entries: [EptEntryT; EPT_ENTRY_COUNT],
}


pub struct EptPml4 {
    pub pml4: [EptEntryT; EPT_ENTRY_COUNT],
    pub pdpt: [EptEntryT; EPT_ENTRY_COUNT],
}
