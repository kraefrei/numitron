// Program for displaying digits on a numitron seven segment display

extern crate rppal;
extern crate chrono;
extern crate ctrlc;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use std::ops::Rem;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rppal::gpio::Gpio;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use chrono::*;

use SevenSegmentNumitronCharacters::*;
const OE: u8 = 23;
const LE: u8 = 22;

#[macro_use]
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Setting up hardware...");
    // Retrieve the GPIO pin and configure it as an output.
    let mut output_enable_pin = Gpio::new()?.get(OE)?.into_output();
    let mut latch_enable_pin = Gpio::new()?.get(LE)?.into_output();
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;
    // Disable driver output while shifting in next number
    output_enable_pin.set_low();
    // Initialize the latch pin low
    latch_enable_pin.set_low();
    let sequence = hashmap![0 => Zero, 1 => One, 2 => Two, 3=> Three, 4=> Four, 5 => Five,
                            6 => Six, 7 => Seven, 8 => Eight, 9 => Nine, 10 => Off];
    /**********************************************************************************************/
    let mut second: u32 = 0;
    let mut minute: u32 = 0;
    let mut hour: u32 = 0;
    let mut sep: u8 = 0xF;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    while (running.load(Ordering::SeqCst)) {
        let mut numitron: Vec<u8> = Vec::new();
        let local: DateTime<Local> = Local::now();
        if (second != local.second()) {
            second = local.second();
            minute = local.minute();
            match sequence.get(&(minute % 10)) {
                Some(x) => {numitron.push(x.clone() as u8);
                            println!("")},
                None => println!("Invalid key from MOD operator for seconds!"),
            }
            match sequence.get(&(minute / 10)) {
                Some(x) => numitron.push(x.clone() as u8),
                None => println!("Invalid key from DIV operator for seconds!"),
            }
            hour = local.hour();
            sep = !sep;
            numitron.push(sep << 4 as u8);
            match sequence.get(&(hour % 10)) {
                Some(x) => numitron.push(x.clone() as u8),
                None => println!("Invalid key from MOD operator for minutes!"),
            }
            if (hour / 10 != 0) {
                match sequence.get(&(hour / 10)) {
                    Some(x) => numitron.push(x.clone() as u8),
                    None => println!("Invalid key from DIV operator for minutes!"),
                }
            } else {
                numitron.push(Off.clone() as u8);
            }
        }
        if (!numitron.is_empty()) {
            for digit in numitron.iter() {
                // Write a digit to the driver
                println!("Writing to register...");
                spi.write(&[digit.clone() as u8])?;
                // Flipping the latch
            }
            println!("Shifting bits...");
            latch_enable_pin.set_high();
            latch_enable_pin.set_low();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    for i in 0..5 {
        spi.write(&[Off.clone() as u8])?;
    }
    println!("Clearing numitrons...");
    latch_enable_pin.set_high();
    latch_enable_pin.set_low();

    Ok(())
}

#[derive(Copy, Clone)]
enum SevenSegmentNumitronCharacters {
    Off = 0x00,
    Zero = 0xB7,
    One = 0x81,
    Two = 0xBA,
    Three = 0x9B,
    Four = 0x8D,
    Five = 0x1F,
    Six = 0x3F,
    Seven = 0x83,
    Eight = 0xBF,
    Nine = 0x9F,
    /*************
     *  *--2--*  *
     *  |     |  *
     *  3     8  *
     *  |     |  *
     *  *--4--*  *
     *  |     |  *
     *  6     1  *
     *  |     |  *
     *  *--5--*  *
     *         7 *
     *************/
}

#[derive(Copy, Clone)]
enum SevenSegmentLEDCharacters {
    Off = 0x00,
    Zero = 0x77,
    One = 0x41,
    Two = 0x3B,
    Three = 0x6B,
    Four = 0x4D,
    Five = 0x6E,
    Six = 0x7E,
    Seven = 0x43,
    Eight = 0x7F,
    Nine = 0x6F,
}

