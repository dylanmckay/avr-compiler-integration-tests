extern crate simavr_sys as simavr;
#[macro_use]
extern crate bitflags;
extern crate tempfile;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

use std::io::prelude::*;
use std::io::{self, stderr};
use std::{env, process};

fn open_firmware() -> Result<sim::Firmware, io::Error> {
    // See if a path was specified on the command line.
    match env::args().skip(1).next() {
        Some(path) => {
            Ok(sim::Firmware::read_elf_via_disk(path).unwrap())
        }
        // Assume standard input.
        None => {
            writeln!(stderr(), "note: no firmware path specified, reading from stdin").unwrap();
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            Ok(sim::Firmware::read_elf(&buffer).unwrap())
        },
    }
}

fn main() {
    let mut avr = sim::Avr::with_name("atmega328").unwrap();

    let firmware = open_firmware().expect("could not open firmware");

    avr.flash(&firmware);
    sim::uart::attach_to_stdout(&mut avr);

    loop {
        match avr.run_cycle() {
            sim::State::Running => (),
            sim::State::Crashed => {
                writeln!(stderr(), "simulation crashed").unwrap();
                process::exit(1);
            },
            // Keep running when in setup,limbo,etc.
            state => {
                println!("state {:?}", state);
                if !state.is_running() {
                    break;
                }
            },
        }
    }
}
