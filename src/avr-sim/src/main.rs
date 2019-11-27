extern crate simavr_sys as simavr;
#[macro_use] extern crate bitflags;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

use clap::{App, Arg};
use std::io::prelude::*;
use std::io::{self, stderr};
use std::{env, process};

fn open_firmware(executable_path: Option<&std::path::Path>) -> Result<sim::Firmware, io::Error> {
    // See if a path was specified on the command line.
    match executable_path {
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

#[derive(Clone, Debug)]
pub struct CommandLine {
    executable_path: Option<std::path::PathBuf>,
    watches: Vec<Watch>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Watch {
    IoRegister {
        name: String,
    },
}

fn parse_cmd_line() -> CommandLine {
    let matches = App::new("avr-sim")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("watch-io-reg")
            .short("w")
            .long("watch-io-reg")
            .value_name("IO REGISTER")
            .help("Watch an IO register (IO REGISTER is like 'PORTB', 'PORTA', etc)")
            .multiple(true)
            .takes_value(true))
        .arg(Arg::with_name("EXECUTABLE PATH")
            .help("A path to the executable file to run. Defaults to standard input if not specified.")
            .required(false)
            .index(1))
        .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))
        .get_matches();

    let mut watches = Vec::new();
    watches.extend(matches.values_of_lossy("watch-io-reg").unwrap_or_else(|| Vec::new()).into_iter().map(|ioreg| Watch::IoRegister { name: ioreg }));

    CommandLine {
        executable_path: matches.value_of("EXECUTABLE PATH").map(Into::into),
        watches,
    }
}

fn main() {
    let command_line = self::parse_cmd_line();
    println!("command line: {:#?}", command_line);


    let mut avr = sim::Avr::with_name("atmega328").unwrap();

    let firmware = open_firmware(command_line.executable_path.as_ref().map(|p| p as _)).expect("could not open firmware");

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
            state if !state.is_running() => break,
            // We don't care about other states.
            _ => (),
        }
    }
}
