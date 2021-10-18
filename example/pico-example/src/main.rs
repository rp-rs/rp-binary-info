//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;
use rp2040_hal as hal;
use rp_binary_info as bi;

use hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

extern "C" {
    static __bi_entries_start: bi::entry::Addr;
    static __bi_entries_end: bi::entry::Addr;
    static __sdata: u32;
    static __edata: u32;
    static __sidata: u32;
}

/// Picotool can find this block in our ELF file and report interesting metadata.
#[link_section = ".bi_header"]
#[used]
pub static PICOTOOL_META: bi::Header =
    unsafe { bi::Header::new(&__bi_entries_start, &__bi_entries_end, &MAPPING_TABLE) };

/// This tells picotool how to convert RAM addresses back into Flash addresses
static MAPPING_TABLE: [bi::MappingTableEntry; 2] = [
    // This is the entry for .data
    bi::MappingTableEntry {
        source_addr_start: unsafe { &__sidata },
        dest_addr_start: unsafe { &__sdata },
        dest_addr_end: unsafe { &__edata },
    },
    // This is the terminating marker
    bi::MappingTableEntry {
        source_addr_start: core::ptr::null(),
        dest_addr_start: core::ptr::null(),
        dest_addr_end: core::ptr::null(),
    },
];

/// This is a list of references to our table entries
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [bi::entry::Addr; 6] = [
    PROGRAM_NAME.addr(),
    PROGRAM_VERSION_STRING.addr(),
    PROGRAM_URL.addr(),
    PROGRAM_BUILD_ATTRIBUTE.addr(),
    PICO_BOARD.addr(),
    BOOT2_NAME.addr(),
];

// PROGRAM_NAME (program_name = value)
// PROGRAM_VERSION_STRING (program_version = value)
// PROGRAM_BUILD_DATE_STRING (program_build_date = value)
// PROGRAM_URL (program_url = value)
// PROGRAM_DESCRIPTION (program_description = value)
// PROGRAM_FEATURE (program_features.push_back(value))
// PROGRAM_BUILD_ATTRIBUTE (build_attributes.push_back(value))
// PICO_BOARD (pico_board = value)
// SDK_VERSION (sdk_version = value)
// BOOT2_NAME (boot2_name = value)

/// This is the name of our program
static PROGRAM_NAME: bi::entry::IdAndString = bi::program_name(concat!(env!("CARGO_PKG_NAME"), "\0"));

/// This is somewhere you can get more info about this program
static PROGRAM_URL: bi::entry::IdAndString = bi::program_url(concat!(env!("CARGO_PKG_HOMEPAGE"), "\0"));
 
/// This is the version of our program
static PROGRAM_VERSION_STRING: bi::entry::IdAndString = bi::program_version_string(
    concat!(env!("CARGO_PKG_VERSION"), " (", env!("BUILD_GIT_VERSION"), ")\0")
    );

// PROGRAM_URL

// PROGRAM_DESCRIPTION

// PROGRAM_FEATURE

/// This is Debug or Release
static PROGRAM_BUILD_ATTRIBUTE: bi::entry::IdAndString = bi::program_build_attribute(concat!(env!("BUILD_PROFILE"), "\0"));

/// This is the board we can run on
static PICO_BOARD: bi::entry::IdAndString = bi::pico_board("Raspberry Pi Pico\0");
    
/// This is Debug or Release
static BOOT2_NAME: bi::entry::IdAndString = bi::boot2_name("W25Q080\0");

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let mut led_pin = pins.gpio25.into_push_pull_output();

    loop {
        info!("on!");
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        info!("off!");
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}
