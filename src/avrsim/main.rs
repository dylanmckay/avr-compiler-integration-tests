extern crate simavr_sys as simavr;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

fn main() {
    let mut avr = sim::Avr::with_name("atmega328").unwrap();
    let firmware = sim::Firmware::read_elf("arduino-uart-loop.elf").unwrap();

    avr.load(&firmware);

    sim::uart::attach(&mut avr);

    loop {
        match avr.run_cycle() {
            sim::State::Running => (),
            state if !state.is_running() => break,
            state => println!("{:?}", state),
        }
    }
}
