//! Prints "Hello, world!" on the OpenOCD console using semihosting
//!
//! ---

#![feature(used)]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate sgp30;
extern crate stm32f103xx_hal as hal;

use core::fmt::Write;

use cortex_m::asm;
use cortex_m_semihosting::hio;
use hal::delay::Delay;
use hal::i2c::{I2c, Mode};
use hal::prelude::*;
use sgp30::Sgp30;

fn main() {
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Hello, world!").unwrap();
    writeln!(stdout, "Initializing peripherals...").unwrap();

    // Get access to device peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = hal::stm32f103xx::Peripherals::take().unwrap();

    // Get access to some registers required for I²C
    let mut rcc = dp.RCC.constrain(); // Reset and clock control register
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2); // Alternate Function I/O register
    let mut flash = dp.FLASH.constrain(); // ?

    // Get access to PORTB pins
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // We want to use pin PB8 as SCL1 and PB9 as SDA1 
    let scl1 = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
    let sda1 = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);

    // Freeze clock configuration
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // Determine frequency / mode
    let mode = Mode::Standard { frequency: 100_000 };

    // Initialize I²C peripheral on pins PB8 / PB9
    let i2c = I2c::i2c1(
        dp.I2C1,
        (scl1, sda1),
        &mut afio.mapr,
        mode,
        clocks,
        &mut rcc.apb1,
    );

    // Initialize system timer (SysTick) as delay provider
    let delay = Delay::new(cp.SYST, clocks);

    writeln!(stdout, "Initializing SGP30 sensor...").unwrap();

    // Initialize SGP30 sensor
    let sgp30 = Sgp30::new(i2c, 0x58, delay);
}

// As we are not using interrupts, we just register a dummy catch all handler
#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [extern "C" fn(); 240] = [default_handler; 240];

extern "C" fn default_handler() {
    asm::bkpt();
}
