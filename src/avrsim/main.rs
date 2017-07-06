extern crate simavr_sys as simavr;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

fn main() {
    let _avr = sim::Avr::with_name("atmega328p").unwrap();
}
