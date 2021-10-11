# rp-binary-info

Lets you embed "Binary Info" in RP2040 Firmware, as readable by [picotool]

[picotool]: https://github.com/raspberrypi/picotool
[pico-sdk]: https://github.com/raspberrypi/pico-sdk

## What is Binary Info?

When you compile firmware for the Raspberry Silicon RP2040 using the official
[pico-sdk], your compiled binaries (in both ELF and UF2 format), contain some
extra metadata. This metadata is known as "Binary Info", and it can be used
to inform the holder of the ELF (or UF2) file various properties. These include,
but are not limited to:

* The program version
* The git revision
* The program name

The format and structure of the Binary Info is defined by
https://github.com/raspberrypi/pico-sdk/tree/master/src/common/pico_binary_info.

## How does this work?

### The Magic Header

When you run [picotool] on your ELF or UF2 file, it scans the first 256 bytes of
your program (it skips the *first* 256 bytes because that contains boot2, and it
looks at the next 256 bytes) for a *Magic Header*.
.
The *Magic Header* five 32-bit values:

* `marker_start` - The value `0x7188ebf2`
* `entries_start` - The address of the start of the entry table
* `entries_end` - The address of the end of the entry table 
* `mapping_table` - The start address of the mapping table
* `marker_end` - the value `0xe71aa390`

### The Entry Table

The *Entry Table* is a list of 32-bit values. Each value is the address of an
Entry. The address may point to either RAM or Flash, although RAM addresses are
converted back to Flash addresses using the *Mapping Table* discussed later.

There is no particular order to the Entry Table. The idea is to set up a Linker
section for this table. The way the [pico-sdk] works is that when you use the
macros to create an Entry as a global variable, the macro also creates a global
variable holding the address of the Entry. That second global variable is tagged
in such as a way that it ends up in the special linker section. The start
address and end address for the section can then be calculated by the linker,
and exported back into the C program as symbols.

We use a similar trick in Rust.

### Entries

Each *Entry* starts with two 32-bit values:

* `data_type` - describes the kind of information (or 'payload') this Entry holds
* `tag` - describes the 'ID authority' responsible for the ID that follows (this
  prevents collisions if you make your own IDs up)

The values for `data_type` (or at least the ones the [pico-sdk] current uses) are:

* `IdAndInt` (5) - the payload is a 32-bit ID and a 32-bit integer value
* `IdAndString` (6) - the payload is a 32-bit ID and a 32-bit pointer to a
  null-terminated string
* `PinsWithFunction` (8) - the payload is a single 32-bit value which describes
  one or more Pins on the RP2040 (e.g. GP0), and the mode those Pins are in
  (e.g. UART TX)
* `PinsWithName` (9) - like `PinsWithFunction`, but followed by a 32-bit pointer
  to a null-terminated string
* `NamedGroup` (10) - starts a new group of Entries, with the payload including
  the 'parent' group ID, and a 'tag', 'ID' and 'label' for the group. 

### Entry Tags

Raspberry Pi have reserved the *tag* 0x5250 (which corresponds to the ASCII
letters `R` and `P`). When this *tag* is used, the following IDs are valid:

| ID         | Name                      | Expected Type | Example                                 |
| ---------- | ------------------------- | ------------- | --------------------------------------- |
| 0x02031c86 | Program Name              | `IdAndString` | "My Program"                            |
| 0x11a9bc3a | Program Version String    | `IdAndString` | "v1.2.3"                                |
| 0x9da22254 | Program Build Date String | `IdAndString` | "Oct 11 2021"                           |
| 0x68f465de | Binary End Address        | `IdAndInt`    | 0x20001234                              |
| 0x1856239a | Program Url               | `IdAndString` | "https://github.com/my_org/my_code.git" |
| 0xb6a07c19 | Program Description       | `IdAndString` | "This is my amazing program"            |
| 0xa1f4b453 | Program Feature           | `IdAndString` | "UART stdin / stdout"                   |
| 0x4275f0d3 | Program Build Attribute   | `IdAndString` | "Debug"                                 |
| 0x5360b3ab | Sdk Version               | `IdAndString` | "1.2.3"                                 |
| 0xb63cffbb | Pico Board                | `IdAndString` | "adafruit_qtpy_rp2040"                  |
| 0x7f8882e1 | Boot2 Name                | `IdAndString` | "boot2_w25q080"                         |

Note that entries of type `PinsWithFunction` and `PinsWithName` don't have an ID
- the data-type is sufficient to describe the entry.

We (and you) are free to select a unique tag and then create your own range of
IDs - however, [picotool] can only decode the `RP` tagged IDs listed above.

### The Mapping Table

When [picotool] looks at your binary, it is effectively looking at a copy of the
Flash memory. However, some of the values referred to in the various Entries may
reside in a global variable in RAM at run-time. Normally your program's start-up
code will copy any default values for these global variables from Flash to RAM
at start-up. Any global variables which have an all-zeroes default value, simply
get initialised with a call to 'memset' instead.

The Mapping Table tells [picotool] which parts of the Flash image get mapped to
which regions of RAM. If it then finds any RAM addresses when looking at the
Entries, it can work backwards to find the equivalent Flash address and load the
appropriate data.

Of course, if you were planning on constructing or modifying your Entry at
run-time, you're out of luck - [picotool] will only ever be able to find the
power-up default value located in Flash.

Each entry in the Mapping Table contains three 32-bit values:

* `source_addr_start` - the start address in Flash
* `dest_addr_start` - the start address in RAM
* `dest_addr_end` - the end address in RAM

You may have noticed that the *Magic Header* only notes the start of this
Mapping Table, and not the end. The reason for this is unclear, but [picotool]
can find the end of the table easily enough by looking for a 'null' entry - one
with zeroes for all the addresses.

## What do I need to do?

You will need to add two extra pieces to your `memory.x` file (as used by
[cortex-m-rt](https://github.com/rust-embedded/cortex-m-rt)).

```ld
SECTIONS {
    /* ### Picotool 'Binary Info' Header Block
     *
     * Picotool only searches the second 256 bytes of Flash for this block, but
     * that's where our vector table is. We squeeze in this block after the
     * vector table, but before .text.
     */
    .bi_header : ALIGN(4)
    {
        KEEP(*(.bi_header));
        /* Keep this block a nice round size */
        . = ALIGN(4);
    } > FLASH
} INSERT BEFORE .text;

/* Move _stext, to make room for our new section */
_stext = ADDR(.bi_header) + SIZEOF(.bi_header);

SECTIONS {
    /* ### Picotool 'Binary Info' Entries
     *
     * Picotool looks through this block (as we have pointers to it in our header) to find interesting information.
     */
    .bi_entries : ALIGN(4)
    {
        /* We put this in the header */
        __bi_entries_start = .;
        /* Here are the entries */
        KEEP(*(.bi_entries));
        /* Keep this block a nice round size */
        . = ALIGN(4);
        /* We put this in the header */
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;
```

You can then include Binary Info entries in your application's `main.rs` file:

```rust
extern "C" {
    static __bi_entries_start: rp_binary_info::entry::Addr;
    static __bi_entries_end: rp_binary_info::entry::Addr;
    static __sdata: u32;
    static __edata: u32;
    static __sidata: u32;
}

/// Picotool can find this block in our ELF file and report interesting metadata.
#[link_section = ".bi_header"]
#[used]
pub static PICOTOOL_META: rp_binary_info::Header =
    unsafe { rp_binary_info::Header::new(&__bi_entries_start, &__bi_entries_end, &MAPPING_TABLE) };

/// This tells picotool how to convert RAM addresses back into Flash addresses
static MAPPING_TABLE: [rp_binary_info::MappingTableEntry; 2] = [
    // This is the entry for .data
    rp_binary_info::MappingTableEntry {
        source_addr_start: unsafe { &__sidata },
        dest_addr_start: unsafe { &__sdata },
        dest_addr_end: unsafe { &__edata },
    },
    // This is the terminating marker
    rp_binary_info::MappingTableEntry {
        source_addr_start: core::ptr::null(),
        dest_addr_start: core::ptr::null(),
        dest_addr_end: core::ptr::null(),
    },
];

/// This is a list of references to our table entries
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [rp_binary_info::entry::Addr; 3] = [
    PROGRAM_NAME.addr(),
    PROGRAM_VERSION.addr(),
    NUMBER_OF_KITTENS.addr(),
];

/// This is the name of our program
static PROGRAM_NAME: rp_binary_info::entry::IdAndString =
    rp_binary_info::program_name(concat!("my stupid tool 2", "\0"));

/// This is the version of our program
static PROGRAM_VERSION: rp_binary_info::entry::IdAndString =
    rp_binary_info::version(concat!(env!("GIT_VERSION"), "\0"));

/// This is just some application-specific random information to test integer support
static NUMBER_OF_KITTENS: rp_binary_info::entry::IdAndInt =
    rp_binary_info::custom_integer(rp_binary_info::make_tag(b'J', b'P'), 0x0000_0001, 0x12345678);

```

## API Stability

Until this crate reaches version 1.0, the API is liable to change. In
particular, we'd quite like a macro which can both create the Entry and stuff
the address of the Entry into the Entry Table.
## Contributing

Contributions are what make the open source community such an amazing place to
be learn, inspire, and create. Any contributions you make are **greatly
appreciated**.

The steps are:

1. Fork the Project by clicking the 'Fork' button at the top of the page.
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Make some changes to the code or documentation.
4. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
5. Push to the Feature Branch (`git push origin feature/AmazingFeature`)
6. Create a [New Pull Request](https://github.com/rp-rs/rp-binary-info/pulls)
7. An admin will review the Pull Request and discuss any changes that may be required.
8. Once everyone is happy, the Pull Request can be merged by an admin, and your work is part of our project!

## Code of Conduct

Contribution to this crate is organized under the terms of the [Rust Code of
Conduct][CoC], and the maintainer of this crate, the [rp-rs team], promises
to intervene to uphold that code of conduct.

[CoC]: CODE_OF_CONDUCT.md
[rp-rs team]: https://github.com/orgs/rp-rs/teams/rp-rs

## License

The contents of this repository are dual-licensed under the _MIT OR Apache
2.0_ License. That means you can chose either the MIT licence or the
Apache-2.0 licence when you re-use this code. See `MIT` or `APACHE2.0` for more
information on each specific licence.

Any submissions to this project (e.g. as Pull Requests) must be made available
under these terms.
