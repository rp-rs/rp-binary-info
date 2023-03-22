//! # Binary Info
//!
//! Data Types and Functions for handling 'Binary Info' metadata in ELF and UF2
//! files. See README.md for more details.

#![no_std]

pub mod entry;

/// This is the 'Binary Info' header block that `picotool` looks for in your
/// UF2 file to give you useful metadata about your program. It should be
/// placed in the first 256 bytes of your program, so use your `memory.x` to
/// insert a section between `.text` and `.vector_table` and put a static
/// value of this type in that section.
#[repr(C)]
pub struct Header {
    /// Must be equal to Picotool::MARKER_START
    marker_start: u32,
    /// The first in our table of pointers to Entries
    entries_start: &'static entry::Addr,
    /// The last in our table of pointers to Entries
    entries_end: &'static entry::Addr,
    /// The first entry in a null-terminated RAM/Flash mapping table
    mapping_table: *const MappingTableEntry,
    /// Must be equal to Picotool::MARKER_END
    marker_end: u32,
}

/// Allows us to tell picotool where values are in the UF2 given their
/// run-time address. The most obvious example is RAM variables, which must
/// be found in the `.data` section of the UF2.
#[repr(C)]
pub struct MappingTableEntry {
    pub source_addr_start: *const u32,
    pub dest_addr_start: *const u32,
    pub dest_addr_end: *const u32,
}

/// This is the set of data types that `picotool` supports.
#[repr(u16)]
pub enum DataType {
    Raw = 1,
    SizedData = 2,
    BinaryInfoListZeroTerminated = 3,
    Bson = 4,
    IdAndInt = 5,
    IdAndString = 6,
    BlockDevice = 7,
    PinsWithFunction = 8,
    PinsWithName = 9,
    PinsWithNames = 10,
}

/// All Raspberry Pi specified IDs have this tag. You can create your own
/// for custom fields.
pub const TAG_RASPBERRY_PI: u16 = make_tag(b'R', b'P');

/// Used to note the program name - use with entry::IdAndString
pub const ID_RP_PROGRAM_NAME: u32 = 0x02031c86;
/// Used to note the program version - use with entry::IdAndString
pub const ID_RP_PROGRAM_VERSION_STRING: u32 = 0x11a9bc3a;
/// Used to note the program build date - use with entry::IdAndString
pub const ID_RP_PROGRAM_BUILD_DATE_STRING: u32 = 0x9da22254;
/// Used to note the size of the binary - use with entry::IdAndInt
pub const ID_RP_BINARY_END: u32 = 0x68f465de;
/// Used to note a URL for the program - use with entry::IdAndString
pub const ID_RP_PROGRAM_URL: u32 = 0x1856239a;
/// Used to note a description of the program - use with entry::IdAndString
pub const ID_RP_PROGRAM_DESCRIPTION: u32 = 0xb6a07c19;
/// Used to note some feature of the program - use with entry::IdAndString
pub const ID_RP_PROGRAM_FEATURE: u32 = 0xa1f4b453;
/// Used to note some whether this was a Debug or Release build - use with entry::IdAndString
pub const ID_RP_PROGRAM_BUILD_ATTRIBUTE: u32 = 0x4275f0d3;
/// Used to note the Pico SDK version used - use with entry::IdAndString
pub const ID_RP_SDK_VERSION: u32 = 0x5360b3ab;
/// Used to note which board this program targets - use with entry::IdAndString
pub const ID_RP_PICO_BOARD: u32 = 0xb63cffbb;
/// Used to note which `boot2` image this program uses - use with entry::IdAndString
pub const ID_RP_BOOT2_NAME: u32 = 0x7f8882e1;

impl Header {
    /// This is the `BINARY_INFO_MARKER_START` magic value from `picotool`
    const MARKER_START: u32 = 0x7188ebf2;
    /// This is the `BINARY_INFO_MARKER_END` magic value from `picotool`
    const MARKER_END: u32 = 0xe71aa390;

    /// Create a new `picotool` compatible header.
    ///
    /// * `entries_start` - the first [`entry::Addr`](binary_info::entry::Addr) in the table
    /// * `entries_end` - the last [`entry::Addr`](binary_info::entry::Addr) in the table
    /// * `mapping_table` - the RAM/Flash address mapping table
    pub const fn new(
        entries_start: &'static entry::Addr,
        entries_end: &'static entry::Addr,
        mapping_table: &'static [MappingTableEntry],
    ) -> Self {
        let mapping_table = mapping_table.as_ptr();
        Self {
            marker_start: Self::MARKER_START,
            entries_start,
            entries_end,
            mapping_table,
            marker_end: Self::MARKER_END,
        }
    }
}

/// Create a 'Binary Info' entry containing the program name
///
/// The given string must be null-terminated, so put a `\0` at the end of
/// it.
pub const fn program_name(name: &'static str) -> entry::IdAndString {
    entry::IdAndString {
        header: entry::Common {
            data_type: DataType::IdAndString,
            tag: TAG_RASPBERRY_PI,
        },
        id: ID_RP_PROGRAM_NAME,
        value: name.as_ptr() as *const u8,
    }
}

/// Create a 'Binary Info' entry containing the program version.
///
/// The given string must be null-terminated, so put a `\0` at the end of
/// it.
pub const fn version(name: &'static str) -> entry::IdAndString {
    entry::IdAndString {
        header: entry::Common {
            data_type: DataType::IdAndString,
            tag: TAG_RASPBERRY_PI,
        },
        id: ID_RP_PROGRAM_VERSION_STRING,
        value: name.as_ptr() as *const u8,
    }
}

/// Create a 'Build Info' entry containing the build date
///
/// The given string must be null-terminated, so put a `\0` at the end of
/// it.
pub const fn build_date(name: &'static str) -> entry::IdAndString {
    entry::IdAndString {
        header: entry::Common {
            data_type: DataType::IdAndString,
            tag: TAG_RASPBERRY_PI,
        },
        id: ID_RP_PROGRAM_BUILD_DATE_STRING,
        value: name.as_ptr() as *const u8,
    }
}

/// Create a 'Binary Info' entry containing a custom integer entry.
pub const fn custom_integer(tag: u16, id: u32, value: u32) -> entry::IdAndInt {
    entry::IdAndInt {
        header: entry::Common {
            data_type: DataType::IdAndInt,
            tag,
        },
        id,
        value,
    }
}

/// Create a 'Binary Info' entry containing a custom string entry.
pub const fn custom_string(tag: u16, id: u32, value: &'static str) -> entry::IdAndString {
    entry::IdAndString {
        header: entry::Common {
            data_type: DataType::IdAndString,
            tag,
        },
        id,
        value: value.as_ptr() as *const u8,
    }
}

/// Create a tag from two ASCII letters.
pub const fn make_tag(c1: u8, c2: u8) -> u16 {
    u16::from_be_bytes([c2, c1])
}

// We need this as rustc complains that is is unsafe to share `*const u32`
// pointers between threads. We only allow these to be created with static
// data, so this is OK.
unsafe impl Sync for Header {}

// We need this as rustc complains that is is unsafe to share `*const u8`
// pointers between threads. We only allow these to be created with static
// string slices, so it's OK.
unsafe impl Sync for entry::IdAndString {}

// We need this as rustc complains that is is unsafe to share `*const u32`
// pointers between threads. We only allow these to be created with static
// data, so this is OK.
unsafe impl Sync for MappingTableEntry {}

// We need this as rustc complains that is is unsafe to share `*const u32`
// pointers between threads. We only allow these to be created with static
// data, so this is OK.
unsafe impl Sync for entry::Addr {}

// End of file
