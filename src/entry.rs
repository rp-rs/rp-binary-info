//! Entries
//! 
//! Types to describe Entries - the objects which are pointed to from the Entry Table.

/// All Entries start with this common header
#[repr(C)]
pub(crate) struct Common {
    pub(crate) data_type: super::DataType,
    pub(crate) tag: u16,
}

/// An entry which contains both an ID (e.g. `ID_RP_PROGRAM_NAME`) and a pointer to a null-terminated string.
#[repr(C)]
pub struct IdAndString {
    pub(crate) header: Common,
    pub id: u32,
    pub value: *const u8,
}

/// An entry which contains both an ID (e.g. `ID_RP_BINARY_END`) and an integer.
#[repr(C)]
pub struct IdAndInt {
    pub(crate) header: Common,
    pub id: u32,
    pub value: u32,
}

/// This is a reference to an entry. It's like a `&dyn` ref to some type `T:
/// Entry`, except that the run-time type information is encoded into the
/// Entry itself in very specific way.
#[repr(transparent)]
pub struct Addr(*const u32);

impl IdAndString {
    /// Get this entry's address
    pub const fn addr(&self) -> Addr {
        Addr(self as *const Self as *const u32)
    }
}

impl IdAndInt {
    /// Get this entry's address
    pub const fn addr(&self) -> Addr {
        Addr(self as *const Self as *const u32)
    }
}
