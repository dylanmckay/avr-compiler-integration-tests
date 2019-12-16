extern crate simavr_sys as simavr;
#[macro_use] extern crate bitflags;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

use clap::{App, Arg};
use std::fmt::Debug;
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
    print_before: Vec<Watch>,
    print_after: Vec<Watch>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Watch {
    DataMemory {
        address: Pointer,
        data_type: DataType,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum DataType {
    Char,
    NullTerminated(Box<DataType>),
    U8,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pointer {
    pub address: u32,
    pub natural_radix: u32,
}

fn parse_cmd_line() -> CommandLine {
    let matches = App::new("avr-sim")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("print-before")
            .long("print-before")
            .value_name("WATCHABLE")
            .help("Print a value before the program completes (but after the chip is flashed)")
            .multiple(true)
            .takes_value(true))
        .arg(Arg::with_name("print-after")
            .short("p")
            .long("print-after")
            .value_name("WATCHABLE")
            .help("Print a value after the program completes")
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

    CommandLine {
        executable_path: matches.value_of("EXECUTABLE PATH").map(Into::into),
        print_before: matches.values_of_lossy("print-before").unwrap_or_else(|| Vec::new()).into_iter().map(|watch| watch.parse().unwrap()).collect(),
        print_after: matches.values_of_lossy("print-after").unwrap_or_else(|| Vec::new()).into_iter().map(|watch| watch.parse().unwrap()).collect(),
    }
}

fn main() {
    let command_line = self::parse_cmd_line();

    let mut avr = sim::Avr::with_name("atmega328").unwrap();

    let firmware = open_firmware(command_line.executable_path.as_ref().map(|p| p as _)).expect("could not open firmware");

    avr.flash(&firmware);
    sim::uart::attach_to_stdout(&mut avr);

    dump_values("before_execution", &command_line.print_before[..], &avr);

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

    dump_values("after_execution", &command_line.print_after[..], &avr);
}

fn dump_values(label: &str,
               watches: &[Watch],
               avr: &sim::Avr) {
    if !watches.is_empty() {
        print_heading(&format!("Dumping all {}", label.replace("_", " ")));

        for watch in watches {
            dump_value(label, watch, &avr);
        }
    }
}

fn dump_value(label: &str,
              watch: &Watch,
              avr: &sim::Avr) {
    let current_value = warn_on_error(&format!("get {:?}", watch), || watch.current_value_as_str(&avr));

    if let Some(current_value) = current_value {
        let is_multi_line = current_value.lines().count() > 1;

        if is_multi_line {
            for (i, line) in current_value.lines().enumerate() {
                let line_details = if is_multi_line { format!("[line {}]", i + 1) } else { "".to_owned() };
                println!("{}({}){} = {}", label, watch.location(), line_details, line);
            }
        } else {
            println!("{}({}) = {}", label, watch.location(), current_value);
        }
    }
}

fn print_heading(heading: &str) {
    println!();
    println!("{}", heading);
    for _ in 0..heading.len() {
        print!("=");
    }
    println!();
    println!();
}

fn warn_on_error<T, E>(what_we_are_doing: &str, f: impl FnOnce() -> Result<T, E>)
    -> Option<T>
    where E: std::fmt::Display {
    match f() {
        Ok(output) => Some(output),
        Err(e) => {
            eprintln!("error: could not {}: {}", what_we_are_doing, e);
            None
        },
    }
}

impl Watch {
    fn current_value_as_str(&self,
        avr: &sim::Avr) -> Result<String, String> {
        match *self {
            Watch::DataMemory { ref address, ref data_type } => {
                let mem = unsafe {
                    let mem_start = (*avr.underlying()).data as *const u8;
                    let mem_size = (*avr.underlying()).ramend as usize;
                    let start_pointer = ((mem_start as usize) + address.address as usize) as *const u8;

                    std::slice::from_raw_parts(start_pointer, mem_size)
                };

                data_type.as_str_from_bytes(mem)
            },
        }
    }

    fn location(&self) -> &impl std::fmt::Display {
        match *self {
            Watch::DataMemory { ref address, .. } => address,
        }
    }
}

impl DataType {
    pub fn as_str_from_bytes(&self, bytes: &[u8]) -> Result<String, String> {
        self.as_str_from_bytes_internal(bytes).map(|(s, _)| s)
    }

    fn as_str_from_bytes_internal<'b>(&self, bytes: &'b [u8]) -> Result<(String, &'b [u8]), String> {
        match *self {
            DataType::U8 => bytes.get(0).map(ToString::to_string).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
            DataType::Char => bytes.get(0).map(|&b| (b as char).to_string()).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
            DataType::NullTerminated(ref element_type) => {
                let mut elements: Vec<String> = Vec::new();

                let bytes_after_null = if let Some(first_null_index) = bytes.iter().position(|&b| b == 0) {
                    let before_and_including_null = &bytes[0..first_null_index + 1];
                    let after_null = &bytes[first_null_index..];

                    let mut left_to_process = before_and_including_null;

                    while left_to_process.len() > 1 { // wait until null empty.
                        let (element, remaining) = element_type.as_str_from_bytes_internal(left_to_process)?;
                        elements.push(element);
                        left_to_process = remaining;
                    }

                    after_null
                } else {
                    &[]
                };

                let formatted_str = match element_type.as_ref() {
                    DataType::Char => format!("{:?}", elements.join("")),
                    _ => format!("{:?}", elements),
                };

                Ok((formatted_str, bytes_after_null))
            },
        }
    }
}

impl std::str::FromStr for Watch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim().to_lowercase();

        if let Some(remaining) = util::try_consume("datamem", &s) {
            let mut equals_char_indices = remaining.match_indices('=').map(|(i, _)| i);

            let (address, data_type): (String, String) = match (equals_char_indices.next(), equals_char_indices.next()) {
                (None, None) => return Err(format!("expected data memory address to include an address and data type separated by equals signs")),
                (Some(_), None) => return Err(format!("expected data memory address to include a data type separated by equals sign")),
                (Some(a), Some(dt)) => (remaining.chars().skip(a + 1).take(dt - a - 1).collect(), remaining.chars().skip(dt + 1).collect()),
                (None, Some(_)) => unreachable!(),
            };

            address.parse().and_then(|address| data_type.parse().map(|dt| (address, dt))).map(|(address, data_type)| {
                Watch::DataMemory { address, data_type }
            })
        } else {
            Err(format!("invalid WATCHABLE: {:?}", s))
        }
    }
}

impl std::str::FromStr for Pointer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();

        let first_char = s.chars().next();
        let second_char = s.chars().skip(1).next();

        if s.chars().all(|c| c.is_digit(10)) {
            u32::from_str_radix(s, 10).map(|address| Pointer { address, natural_radix: 10 }).map_err(|e| format!("could not parse base-10 integer ({:?}) as pointer: {}", s, e))
        } else if first_char == Some('0') && (second_char == Some('x') || second_char == Some('X')) {
            let (_, hex_digits) = s.split_at(2);
            u32::from_str_radix(hex_digits, 16).map(|address| Pointer { address, natural_radix: 16 }).map_err(|e| format!("could not parse base-16 integer ({:?}) as pointer: {}", hex_digits, e))
        } else {
            Err(format!("invalid pointer value: {:?}", s))
        }
    }
}

impl std::str::FromStr for DataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s.trim() {
            "u8" => Ok(DataType::U8),
            "char" => Ok(DataType::Char),
            s => {
                if let Some(inner) = util::try_consume("null_terminated", s) {
                    if let Some(inner) = util::try_consume("=", inner) {
                        DataType::from_str(inner).map(|dt| DataType::NullTerminated(Box::new(dt)))
                    } else {
                        Err(format!("null terminated types must have an inner type separated by the equals sign: {:?}", inner))
                    }
                } else {
                    Err(format!("invalid data type: {:#?}", s))
                }
            },
        }
    }
}

mod util {
    /// Consume the desired string and return the remainder.
    pub fn try_consume<'h>(desired: &str, haystack: &'h str)
        -> Option<&'h str> {
        if haystack.starts_with(desired) {
            let (_, remaining) = haystack.split_at(desired.len());
            Some(remaining)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Pointer {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.natural_radix {
            10 => write!(fmt, "{}", self.address),
            16 => write!(fmt, "0x{:x}", self.address),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_data_memory_address() {
        assert_eq!(Ok(Watch::DataMemory {
            address: Pointer { address: 999, natural_radix: 10 },
            data_type: DataType::U8,
        }), "datamem=999=u8".parse());

        assert_eq!(Ok(Watch::DataMemory {
            address: Pointer { address: 999, natural_radix: 10 },
            data_type: DataType::Char,
        }), "datamem=999=char".parse());

        assert_eq!(Ok(Watch::DataMemory {
            address: Pointer { address: 1, natural_radix: 16 },
            data_type: DataType::NullTerminated(Box::new(DataType::Char)),
        }), "datamem=0x01=null_terminated=char".parse());
    }
}
