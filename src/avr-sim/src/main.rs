extern crate simavr_sys as simavr;
#[macro_use] extern crate bitflags;

/// A high level wrapper over `simavr-sys`.
pub mod sim;

use clap::{App, Arg};
use std::fmt::Debug;
use std::io::prelude::*;
use std::io::{self, stderr};
use std::{env, process, collections::BTreeMap};

const DEFAULT_GDB_PORT: u16 = 1234;

fn open_firmware(executable_path: Option<&std::path::Path>) -> Result<(sim::Firmware, Vec<u8>), io::Error> {
    // See if a path was specified on the command line.
    match executable_path {
        Some(path) => {
            Ok((sim::Firmware::read_elf_via_disk(path).unwrap(), std::fs::read(path).unwrap()))
        }
        // Assume standard input.
        None => {
            writeln!(stderr(), "note: no firmware path specified, reading from stdin").unwrap();
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            Ok((sim::Firmware::read_elf(&buffer).unwrap(), buffer))
        },
    }
}

#[derive(Clone, Debug)]
pub struct CommandLine {
    executable_path: Option<std::path::PathBuf>,
    print_before: Vec<Watch>,
    print_on_change: Vec<Watch>,
    print_after: Vec<Watch>,
    gdb_server_port: Option<u16>,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum MemorySpace {
    Program,
    Data,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Watch {
    MemoryAddress {
        space: MemorySpace,
        address: Pointer,
        data_type: DataType,
    },
    Symbol {
        name: String,
        data_type: DataType,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum DataType {
    Char,
    NullTerminated(Box<DataType>),
    U8,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
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
        .arg(Arg::with_name("print-on-change")
            .long("print-on-change")
            .short("w")
            .value_name("WATCHABLE")
            .help("Print a value whenever it is traced. Watch all changes made on it.")
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
        .arg(Arg::with_name("gdb")
            .long("gdb")
            .help(&format!("Starts the simulator with a GDB server and pauses the program until the debugger instructs continue. The server will be started on port {}", DEFAULT_GDB_PORT)))
        .arg(Arg::with_name("v")
           .short("v")
           .multiple(true)
           .help("Sets the level of verbosity"))
        .after_help(include_str!("../doc/cli_extended_help.txt"))
        .get_matches();

    CommandLine {
        executable_path: matches.value_of("EXECUTABLE PATH").map(Into::into),
        print_before: matches.values_of_lossy("print-before").unwrap_or_else(|| Vec::new()).into_iter().map(|watch| watch.parse().unwrap()).collect(),
        print_on_change: matches.values_of_lossy("print-on-change").unwrap_or_else(|| Vec::new()).into_iter().map(|watch| watch.parse().unwrap()).collect(),
        print_after: matches.values_of_lossy("print-after").unwrap_or_else(|| Vec::new()).into_iter().map(|watch| watch.parse().unwrap()).collect(),
        gdb_server_port: if matches.is_present("gdb") { Some(DEFAULT_GDB_PORT) } else { None },
    }
}

fn fetch_only_section_of_kind<'data, 'obj>(
    section_kind: object::SectionKind,
    object: &'obj object::read::File<'data>,
) -> Result<object::read::Section<'data, 'obj>, String> {
    use object::read::{Object, ObjectSection};

    let matching_sections = object.sections().filter(|s| {
        s.kind() == section_kind
    }).collect::<Vec<_>>();

    match matching_sections.len() {
        0 => Err(format!("there is no ELF section of kind {:?}", section_kind)),
        1 => Ok(matching_sections.into_iter().next().unwrap()),
        _ => {
            let section_names = matching_sections.iter().map(|s| s.name().map(ToOwned::to_owned).unwrap_or(String::new())).collect::<Vec<_>>();
            Err(format!("there is more than one {:?} section (names: {})", section_kind, section_names.join(", ")))
        },
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub struct WatchableSymbol {
    pub name: String,
    pub memory_space: MemorySpace,
    /// The pointer as relative to the start of the memory space.
    pub address: Pointer,
}

// TODO: it should be possible to ask for the list of these from the command line.
fn parse_watchable_symbols_from_elf(elf_data: &[u8], avr: &sim::Avr) -> Vec<WatchableSymbol> {
    use object::read::{Object, ObjectSection};

    let object = object::read::File::parse(elf_data).unwrap();
    let mut watchables = Vec::new();

    let text_section = fetch_only_section_of_kind(object::SectionKind::Text, &object).unwrap();
    let data_section = fetch_only_section_of_kind(object::SectionKind::Data, &object).unwrap();

    println!("text address: {:?}", text_section.address());
    println!("data address: {:?}", data_section.address());

    'symbols: for (_, symbol) in object.symbols() {
        let symbol_name = if let Some(name) = symbol.name() {
            name.trim()
        } else {
            continue 'symbols; // skip nameless symbols
        };

        if symbol_name.is_empty() {
            continue 'symbols; // skip symbols with empty name.
        }

        let (parent_section, memory_space, extra_offset) = if symbol.section_index() == Some(text_section.index()) {
            (&text_section, MemorySpace::Program, 0)
        } else if symbol.section_index() == Some(data_section.index()) {
            let data_section_load_start = avr.raw().ioend as u32 + 1;
            (&data_section, MemorySpace::Data, data_section_load_start)
        } else {
            continue 'symbols; // skip symbols not in text or data sections
        };

        let relative_address = Pointer { address: extra_offset + (symbol.address() - parent_section.address()) as u32, natural_radix: 16 };
        watchables.push(WatchableSymbol {
            name: symbol_name.to_owned(), memory_space, address: relative_address,
        });
    }

    watchables
}

fn main() {
    let command_line = self::parse_cmd_line();

    let mut avr = sim::Avr::with_name("atmega328").unwrap();

    let (firmware, firmware_buffer) = open_firmware(command_line.executable_path.as_ref().map(|p| p as _)).expect("could not open firmware");

    avr.flash(&firmware);
    sim::uart::attach_to_stdout(&mut avr);

    // NOTE: this should happen after the AVR program is flashed.
    let watchable_symbols = parse_watchable_symbols_from_elf(&firmware_buffer, &avr);
    println!("possible watches: {:?}", watchable_symbols);

    if let Some(gdb_port) = command_line.gdb_server_port {
        avr.raw_mut().gdb_port = gdb_port as i32;
        avr.raw_mut().state = simavr_sys::cpu_Stopped as _;

        unsafe { simavr_sys::avr_gdb_init(avr.raw_mut()); }

        println!("GDB server enabled on port {}", gdb_port);
        println!("NOTE: the MCU will be started in a paused state such that you must continue the initial execution in a debugger");

        let mut example_gdb_command = format!("avr-gdb --eval-command 'target remote localhost:{}'", gdb_port);
        if let Some(firmware_path) = command_line.executable_path.as_ref() {
            example_gdb_command += &format!(" '{}'", firmware_path.display());
        }

        println!();
        println!("GDB example:");
        println!();
        println!("  {}", example_gdb_command);
    }


    dump_values("before_execution", &command_line.print_before[..], &watchable_symbols, &avr);

    let mut prior_values_watched_onchange = get_current_values(&command_line.print_on_change[..], &watchable_symbols, &avr);
    let mut current_cycle_number: u64 = 0;

    loop {
        current_cycle_number += 1;
        // println!("tick {}", current_cycle_number);
        let sim_state =  avr.run_cycle();

        dump_onchanged_watches(&mut prior_values_watched_onchange, &command_line, &watchable_symbols, &avr, current_cycle_number);


        match sim_state {
            sim::State::Running | sim::State::Stopped => (),
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

    dump_onchanged_watches(&mut prior_values_watched_onchange, &command_line, &watchable_symbols, &avr, current_cycle_number);

    dump_values("after_execution", &command_line.print_after[..], &watchable_symbols, &avr);
}

fn dump_onchanged_watches(
    prior_values_watched_onchange: &mut BTreeMap<Watch, String>,
    command_line: &CommandLine,
    watchable_symbols: &[WatchableSymbol],
    avr: &sim::Avr,
    current_cycle_number: u64,
) {
    let changed_watches = {
        let current_values_watched_onchange = self::get_current_values(&command_line.print_on_change[..], watchable_symbols, &avr);
        let changed_watches = self::get_changed_watches(prior_values_watched_onchange, &current_values_watched_onchange);

        *prior_values_watched_onchange = current_values_watched_onchange;
        changed_watches
    };

    if !changed_watches.is_empty() {
        print_heading(&format!("Dumping watches values changed in CPU cycle #{}", current_cycle_number));

        for (watch, current_value) in changed_watches {
            dump_value("changed", &watch, &current_value);
        }
    }

}

fn get_changed_watches(before: &BTreeMap<Watch, String>, after: &BTreeMap<Watch, String>)
    -> BTreeMap<Watch, String> {
    assert_eq!(before.len(), after.len());

    after.iter().filter_map(|(watch, current_value)| {
        let prior_value = before.get(watch).expect("watched variable has no stored prior state");

        if current_value == prior_value {
            None
        } else {
            Some((watch.clone(), current_value.clone()))
        }
    }).collect()
}

fn dump_values(label: &str,
               watches: &[Watch],
               watchable_symbols: &[WatchableSymbol],
               avr: &sim::Avr) {
    if !watches.is_empty() {
        print_heading(&format!("Dumping all {}", label.replace("_", " ")));

        for watch in watches {
            dump_watch(label, watch, watchable_symbols, avr);
        }
    }
}

/// Gets the current values of the given watches.
fn get_current_values(watches: &[Watch], watchable_symbols: &[WatchableSymbol], avr: &sim::Avr) -> BTreeMap<Watch, String> {
    watches.iter().flat_map(|watch| {
        warn_on_error(&format!("get {:?}", watch), || watch.current_value_as_str(&avr, watchable_symbols)).map(|value| (watch.clone(), value))
    }).collect()
}

fn dump_watch(label: &str,
              watch: &Watch,
              watchable_symbols: &[WatchableSymbol],
              avr: &sim::Avr) {
    let current_value = warn_on_error(&format!("get {:?}", watch), || watch.current_value_as_str(&avr, watchable_symbols));

    if let Some(current_value) = current_value {
        self::dump_value(label, watch, &current_value);
    }
}

fn dump_value(label: &str,
              watch: &Watch,
              current_value: &str) {
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
        avr: &sim::Avr,
        watchable_symbols: &[WatchableSymbol],
    ) -> Result<String, String> {
        match *self {
            Watch::MemoryAddress { space, address, ref data_type } => {
                self::read_current_memory_address(space, address, data_type, avr)
            },
            Watch::Symbol { ref name, ref data_type } => {
                match watchable_symbols.iter().find(|s| s.name == *name) {
                    Some(WatchableSymbol { memory_space, address, .. }) => {
                        self::read_current_memory_address(*memory_space, address.clone(), data_type, avr)
                    },
                    None => Err(format!("the symbol '{}' does not exist in the ELF file, or the ELF file contains no debug information", name)),
                }
            },
        }
    }

    fn location(&self) -> String {
        match *self {
            Watch::MemoryAddress { ref space, ref address, .. } => format!("{} ({})", address, space.human_label()),
            Watch::Symbol { ref name, .. } => format!("{} (symbol)", name)
        }
    }
}

fn read_current_memory_address(
    space: MemorySpace,
    address: Pointer,
    data_type: &DataType,
    avr: &sim::Avr,
) -> Result<String, String> {
    let (memory_space_start_host_ptr, memory_space_size) = match space {
        MemorySpace::Data => {
            let (data_space_start, data_space_size) = unsafe {
                ((*avr.underlying()).data as *const u8, (*avr.underlying()).ramend as usize) // N.B. 'ramend' really is the size. misnomer.
            };

            (data_space_start, data_space_size)
        },
        MemorySpace::Program => unimplemented!("watches on program space"),
    };

    let memory_space_byte_slice = unsafe { std::slice::from_raw_parts(memory_space_start_host_ptr, memory_space_size) };

    data_type.as_str_from_bytes(&memory_space_byte_slice[address.address as usize..])
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
        let s = s.trim();

        if let Some(remaining) = util::try_consume("datamem", &s) {
            let mut equals_char_indices = remaining.match_indices('=').map(|(i, _)| i);

            let (address, data_type): (String, String) = match (equals_char_indices.next(), equals_char_indices.next()) {
                (None, None) => return Err(format!("expected data memory address to include an address and data type separated by equals signs")),
                (Some(_), None) => return Err(format!("expected data memory address to include a data type separated by equals sign")),
                (Some(a), Some(dt)) => (remaining.chars().skip(a + 1).take(dt - a - 1).collect(), remaining.chars().skip(dt + 1).collect()),
                (None, Some(_)) => unreachable!(),
            };

            address.parse().and_then(|address| data_type.parse().map(|dt| (address, dt))).map(|(address, data_type)| {
                Watch::MemoryAddress { address, data_type, space: MemorySpace::Data }
            })
        } else if s.chars().filter(|&c| c == '=').count() >= 1 { // symbol name watchables only have one equals sign
            let (symbol_name, data_type_str) = s.split_at(s.find("=").unwrap());
            let data_type_str = &data_type_str[1..];

            data_type_str.parse().map(|data_type| {
                Watch::Symbol { name: symbol_name.to_owned(), data_type }
            })
        } else {
            Err(format!("invalid WATCHABLE: {}", s))
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

impl MemorySpace {
    pub fn human_label(&self) -> &'static str {
        match *self {
            MemorySpace::Program => "program memory",
            MemorySpace::Data => "data memory",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_data_memory_address() {
        assert_eq!(Ok(Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 999, natural_radix: 10 },
            data_type: DataType::U8,
        }), "datamem=999=u8".parse());

        assert_eq!(Ok(Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 999, natural_radix: 10 },
            data_type: DataType::Char,
        }), "datamem=999=char".parse());

        assert_eq!(Ok(Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 1, natural_radix: 16 },
            data_type: DataType::NullTerminated(Box::new(DataType::Char)),
        }), "datamem=0x01=null_terminated=char".parse());
    }

    #[test]
    fn can_parse_watchable_symbol() {
        assert_eq!(Ok(Watch::Symbol {
            name: "TEST_BUFFER".to_owned(),
            data_type: DataType::U8,
        }), "TEST_BUFFER=u8".parse());
    }
}
