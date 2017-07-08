extern crate simavr_sys as simavr;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

use std::io::prelude::*;
use std::io::stderr;
use std::process;

fn main() {
    let mut avr = sim::Avr::with_name("atmega328").unwrap();
    // let firmware = sim::Firmware::read_elf("arduino-uart-loop.elf").unwrap();
    let firmware = sim::Firmware::read_elf("arduino-uart-single.elf").unwrap();

    avr.load(&firmware);

    sim::uart::attach_to_stdout(&mut avr);

    loop {
        match avr.run_cycle() {
            sim::State::Running => (),
            sim::State::Crashed => {
                writeln!(stderr(), "simulation crashed").unwrap();
                process::exit(1);
            },
            state if !state.is_running() => break,
            // Keep running when in setup,limbo,etc.
            e => println!("state {:?}", e),
        }
    }
}
