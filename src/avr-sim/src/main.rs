extern crate simavr_sim as simavr;
#[macro_use] extern crate bitflags;

mod avr_print;

use byteorder::ByteOrder as _;
use clap::{App, Arg};
use std::fmt::Debug;
use std::io::prelude::*;
use std::io::{self, stderr};
use std::{env, process, collections::BTreeMap};

const DEFAULT_GDB_PORT: u16 = 1234;

type ByteOrder = byteorder::LittleEndian;

fn open_firmware(executable_path: Option<&std::path::Path>) -> Result<(simavr::Firmware, Vec<u8>), io::Error> {
    // See if a path was specified on the command line.
    match executable_path {
        Some(path) => {
            Ok((simavr::Firmware::read_elf_via_disk(path).unwrap(), std::fs::read(path).unwrap()))
        }
        // Assume standard input.
        None => {
            writeln!(stderr(), "note: no firmware path specified, reading from stdin").unwrap();
            let mut buffer = Vec::new();
            io::stdin().read_to_end(&mut buffer)?;
            Ok((simavr::Firmware::read_elf(&buffer).unwrap(), buffer))
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
    IoPort { port_letter: char, port_index: Option<u8> },
    IoPin { port_letter: char, port_index: Option<u8> },
    IoDataDirectionRegister { port_letter: char, port_index: Option<u8> },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum DataType {
    Char,
    NullTerminated(Box<DataType>),
    U8, U16, U32, U64, U128,
    I8, I16, I32, I64, I128,
    HighLowBit,
    IoRegisterStatus,
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
        .arg(Arg::with_name("watch")
            .long("watch")
            .short("w")
            .value_name("WATCHABLE")
            .help("Verbosely logs all updates to the given watchable (alias of '--print-before-after <WATCHABLE> --print-on-change <WATCHABLE>')")
            .multiple(true)
            .takes_value(true))
        .arg(Arg::with_name("print-before")
            .long("print-before")
            .value_name("WATCHABLE")
            .help("Print a value before the program completes (but after the chip is flashed)")
            .multiple(true)
            .takes_value(true))
        .arg(Arg::with_name("print-on-change")
            .long("print-on-change")
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
        .arg(Arg::with_name("print-before-after")
            .long("print-before-after")
            .value_name("WATCHABLE")
            .help("Print a value both before the program starts and after the program completes")
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

    let parse_watches = |arg_name: &str| {
        matches.values_of_lossy(arg_name).unwrap_or_else(|| Vec::new()).into_iter().flat_map(|watch| parse_watch(&watch).unwrap()).collect::<Vec<Watch>>()
    };

    let print_on_everything = parse_watches("watch");
    let print_before_after = parse_watches("print-before-after").into_iter().chain(print_on_everything.clone()).collect::<Vec<Watch>>();

    let print_before = parse_watches("print-before").into_iter().chain(print_before_after.clone()).collect::<Vec<Watch>>();
    let print_after = parse_watches("print-after").into_iter().chain(print_before_after.clone()).collect::<Vec<Watch>>();
    let print_on_change = parse_watches("print-on-change").into_iter().chain(print_on_everything).collect::<Vec<Watch>>();

    CommandLine {
        executable_path: matches.value_of("EXECUTABLE PATH").map(Into::into),
        gdb_server_port: if matches.is_present("gdb") { Some(DEFAULT_GDB_PORT) } else { None },
        print_before, print_on_change, print_after,
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
fn parse_watchable_symbols_from_elf(elf_data: &[u8], avr: &simavr::Avr) -> Vec<WatchableSymbol> {
    use object::read::{Object, ObjectSection};

    let object = object::read::File::parse(elf_data).unwrap();
    let mut watchables = Vec::new();

    let text_section = fetch_only_section_of_kind(object::SectionKind::Text, &object).unwrap();
    let data_section = fetch_only_section_of_kind(object::SectionKind::Data, &object).unwrap();

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
    let original_command_line = self::parse_cmd_line();
    let mut command_line = original_command_line.clone();

    let mut avr = simavr::Avr::new("atmega328").unwrap();

    let (firmware, firmware_buffer) = open_firmware(command_line.executable_path.as_ref().map(|p| p as _)).expect("could not open firmware");

    avr.flash(&firmware);
    simavr::uart::attach_to_stdout(&mut avr);

    // NOTE: this always needs to happen after the AVR program is flashed.
    let watchable_symbols = parse_watchable_symbols_from_elf(&firmware_buffer, &avr);
    print_warnings_for_unresolved_watches(&mut command_line, &watchable_symbols);

    let print_config = match avr_print::Config::new(&watchable_symbols) {
        Ok(config) => Some(config),
        Err(message) => {
            eprintln!("warning: cannot intercept and print the libavrlit debug stream: {}.", message);
            None
        },
    };

    if let Some(gdb_port) = command_line.gdb_server_port {
        avr.raw_mut().gdb_port = gdb_port as i32;
        avr.raw_mut().state = simavr::sys::cpu_Stopped as _;

        unsafe { simavr::sys::avr_gdb_init(avr.raw_mut()); }

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

    loop {
        let current_cycle_number = avr.raw().run_cycle_count;

        let sim_state =  avr.run_cycle();

        if let Some(print_config) = print_config.as_ref() {
            if let Some(c) = print_config.consume_character(&avr).expect("could not read from libavrlit debug stream") {
                print!("{}", c);
            }
        }

        dump_onchanged_watches(&mut prior_values_watched_onchange, &command_line, &watchable_symbols, &avr, current_cycle_number);

        match sim_state {
            simavr::State::Running | simavr::State::Stopped => (),
            simavr::State::Crashed => {
                writeln!(stderr(), "simulation crashed").unwrap();
                process::exit(1);
            },
            // Keep running when in setup,limbo,etc.
            state if !state.is_running() => break,
            // We don't care about other states.
            _ => (),
        }
    }

    dump_onchanged_watches(&mut prior_values_watched_onchange, &command_line, &watchable_symbols, &avr, avr.raw().run_cycle_count);

    dump_values("after_execution", &command_line.print_after[..], &watchable_symbols, &avr);
}

fn dump_onchanged_watches(
    prior_values_watched_onchange: &mut BTreeMap<Watch, WatchState>,
    command_line: &CommandLine,
    watchable_symbols: &[WatchableSymbol],
    avr: &simavr::Avr,
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

fn get_changed_watches(before: &BTreeMap<Watch, WatchState>, after: &BTreeMap<Watch, WatchState>)
    -> BTreeMap<Watch, WatchState> {
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
               avr: &simavr::Avr) {
    if !watches.is_empty() {
        print_heading(&format!("Dumping all {}", label.replace("_", " ")));

        for watch in watches {
            dump_watch(label, watch, watchable_symbols, avr);
        }
    }
}

/// Gets the current values of the given watches.
fn get_current_values(watches: &[Watch], watchable_symbols: &[WatchableSymbol], avr: &simavr::Avr) -> BTreeMap<Watch, WatchState> {
    watches.iter().flat_map(|watch| {
        warn_on_error(&format!("get {:?}", watch), || watch.current_value(&avr, watchable_symbols)).map(|value| (watch.clone(), value))
    }).collect()
}

fn dump_watch(label: &str,
              watch: &Watch,
              watchable_symbols: &[WatchableSymbol],
              avr: &simavr::Avr) {
    let current_value = warn_on_error(&format!("get {:?}", watch), || watch.current_value(&avr, watchable_symbols));

    if let Some(current_value) = current_value {
        self::dump_value(label, watch, &current_value);
    }
}

fn dump_value(label: &str,
              watch: &Watch,
              current_value: &WatchState) {
    let current_value = current_value.to_string();
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

fn print_warnings_for_unresolved_watches(
    command_line: &mut CommandLine,
    watchable_symbols: &[WatchableSymbol],
) {
    let watchlists = vec![
        &mut command_line.print_before,
        &mut command_line.print_after,
        &mut command_line.print_on_change,
    ];

    let mut unique_watches = watchlists.iter().flat_map(|w| w.iter()).collect::<Vec<_>>();
    unique_watches.sort();
    unique_watches.dedup();

    // Identify and warn about missing watches.
    let missing_watches = unique_watches.into_iter().filter(|w| {
        match *w {
            Watch::Symbol { ref name, .. } => {
                if let Some(..) = watchable_symbols.iter().find(|s| s.name == *name) {
                    false
                } else {
                    eprintln!("the symbol '{}' does not exist in the ELF file, or the ELF file contains no debug information", name);
                    true
                }
            },
            Watch::MemoryAddress { .. } => false, // technically, we could do range checks here.
            Watch::IoPort { .. } |
                Watch::IoPin { .. } |
                Watch::IoDataDirectionRegister { .. } => false, // technically, we could check that the device supports this port.
        }
    }).cloned().collect::<Vec<_>>();

    // Remove missing watches from watchlists so they do not cause errors.
    for watchlist in watchlists.into_iter() {
        *watchlist = std::mem::replace(watchlist, Vec::new()).into_iter().filter(|watch| {
            !missing_watches.contains(&watch)
        }).collect();
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
    fn current_value(&self,
        avr: &simavr::Avr,
        watchable_symbols: &[WatchableSymbol],
    ) -> Result<WatchState, String> {
        fn read_io_port(port_letter: char, port_index: Option<u8>, avr: &simavr::Avr, f: impl FnOnce(IoState) -> u8) -> Result<WatchState, String> {
            let io_state = read_io_state(port_letter, avr);
            let relevant_value = f(io_state);

            match port_index {
                Some(i) => {
                    let mask = (0b1) << i;
                    Ok(WatchState::HighLowBit(relevant_value & mask == mask))
                },
                None => {
                    Ok(WatchState::IoRegisterStatus(relevant_value))
                },
            }
        }

        match *self {
            Watch::MemoryAddress { space, address, ref data_type } => {
                let bytes = self::read_current_memory_address(space, address, avr)?;
                data_type.as_watch_state_from_bytes(bytes)
            },
            Watch::Symbol { ref name, ref data_type } => {
                let bytes = match watchable_symbols.iter().find(|s| s.name == *name) {
                    Some(WatchableSymbol { memory_space, address, .. }) => {
                        self::read_current_memory_address(*memory_space, address.clone(), avr)?
                    },
                    None => return Err(format!("the symbol '{}' does not exist in the ELF file, or the ELF file contains no debug information", name)),
                };
                data_type.as_watch_state_from_bytes(bytes)
            },
            Watch::IoPort { port_letter, port_index } => {
                read_io_port(port_letter, port_index, avr, |s| s.port)
            },
            Watch::IoPin { port_letter, port_index } => {
                read_io_port(port_letter, port_index, avr, |s| s.pin)
            },
            Watch::IoDataDirectionRegister { port_letter, port_index } => {
                read_io_port(port_letter, port_index, avr, |s| s.data_direction_register)
            },
        }

    }

    fn location(&self) -> String {
        match *self {
            Watch::MemoryAddress { ref space, ref address, .. } => format!("{} ({})", address, space.human_label()),
            Watch::Symbol { ref name, .. } => name.to_owned(),
            Watch::IoPort { port_letter, port_index } => format!("IO PORT{}{}", port_letter, if let Some(i) = port_index { i.to_string() } else { String::new() }),
            Watch::IoPin { port_letter, port_index } => format!("IO PIN{}{}", port_letter, if let Some(i) = port_index { i.to_string() } else { String::new() }),
            Watch::IoDataDirectionRegister { port_letter, port_index } => format!("IO DDR{}{}", port_letter, if let Some(i) = port_index { i.to_string() } else { String::new() }),
        }
    }
}

/// Gets an immutable byte slice starting at the specified AVR memory address.
fn read_current_memory_address<'avr>(
    space: MemorySpace,
    address: Pointer,
    avr: &'avr simavr::Avr,
) -> Result<&'avr [u8], String> {
    let (memory_space_start_host_ptr, memory_space_size) = memory_space_slice_parts(space, avr);
    let memory_space_byte_slice = unsafe { std::slice::from_raw_parts(memory_space_start_host_ptr, memory_space_size) };

    Ok(&memory_space_byte_slice[address.address as usize..])
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchState {
    Char(char),
    Array { elements: Vec<WatchState>, data_type: DataType },
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    HighLowBit(bool),
    IoRegisterStatus(u8),
}

impl std::fmt::Display for WatchState {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            WatchState::Char(c) => std::fmt::Display::fmt(&c, fmt),
            WatchState::U8(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::U16(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::U32(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::U64(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::U128(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::I8(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::I16(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::I32(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::I64(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::I128(i) => std::fmt::Display::fmt(&i, fmt),
            WatchState::Array { ref elements, ref data_type } => {
                let formatted_str = if *data_type == DataType::Char {
                    let chars = elements.iter().map(|e| if let WatchState::Char(c) = e { c } else { unreachable!() });
                    format!("{:?}", chars.collect::<String>())
                } else {
                    format!("{:?}", elements)
                };

                write!(fmt, "{}", formatted_str)
            },
            WatchState::HighLowBit(b) => {
                let label = if b { "HIGH" } else { "LOW" };
                write!(fmt, "{}", label)
            },
            WatchState::IoRegisterStatus(r) => {
                for i in 0..8 {
                    let is_set = r & (1<<i) == (1<<i);
                    write!(fmt, "{}: {}", i, if is_set { "HIGH" } else { " LOW" })?;
                    write!(fmt, ", ")?;
                }

                Ok(())
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IoState {
    port: u8,
    pin: u8,
    data_direction_register: u8,
}

const fn avr_ioctl_def(a: char, b: char, c: char, d: char) -> u32 {
    ((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8)| ((d as u32))
}

const fn avr_ioctl_ioport_get_state(port_name: char) -> u32 {
    avr_ioctl_def('i', 'o', 's', port_name)
}

fn read_io_state(port_letter: char, avr: &simavr::Avr) -> IoState {
    let mut state: simavr::sys::avr_ioport_state_t = unsafe { std::mem::zeroed() };

    let result = unsafe {
        simavr::sys::avr_ioctl(avr.underlying(), avr_ioctl_ioport_get_state(port_letter), &mut state as *mut _ as *mut libc::c_void)
    };

    if result != 0 {
        panic!("avr ioctl failed for port '{}'", port_letter);
    }

    IoState {
        port: state.port() as u8,
        pin: state.pin() as u8,
        data_direction_register: state.ddr() as u8,
    }
}

/// Gets a mutable byte slice starting at the specified AVR memory address.
fn read_current_memory_address_mut<'avr>(
    space: MemorySpace,
    address: Pointer,
    avr: &'avr simavr::Avr,
) -> Result<&'avr mut [u8], String> {
    let (memory_space_start_host_ptr, memory_space_size) = memory_space_slice_parts(space, avr);
    let memory_space_byte_slice = unsafe { std::slice::from_raw_parts_mut(memory_space_start_host_ptr as *mut _, memory_space_size) };

    Ok(&mut memory_space_byte_slice[address.address as usize..])
}

fn memory_space_slice_parts<'avr>(space: MemorySpace, avr: &'avr simavr::Avr) -> (*const u8, usize) {
    match space {
        MemorySpace::Data => {
            let (data_space_start, data_space_size) = unsafe {
                ((*avr.underlying()).data as *const u8, (*avr.underlying()).ramend as usize) // N.B. 'ramend' really is the size. misnomer.
            };

            (data_space_start, data_space_size)
        },
        MemorySpace::Program => unimplemented!("watches on program space"),
    }
}


impl DataType {
    fn as_watch_state_from_bytes(&self, bytes: &[u8]) -> Result<WatchState, String> {
        self.as_watch_state_from_bytes_internal(bytes).map(|(s, _)| s)
    }

    fn as_watch_state_from_bytes_internal<'b>(&self, bytes: &'b [u8]) -> Result<(WatchState, &'b [u8]), String> {
        let parse_number = |byte_count: usize, interpret_bytes: fn(&[u8]) -> WatchState| {
            bytes.get(0..byte_count).ok_or("end of memory".to_string())
                .map(interpret_bytes)
                .map(|s| (s, &bytes[byte_count..]))
        };

        match *self {
            DataType::U8 => bytes.get(0).cloned().map(WatchState::U8).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
            DataType::I8 => bytes.get(0).map(|&b| WatchState::I8(b as i8)).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
            DataType::U16 => parse_number(2, |bytes| WatchState::U16(ByteOrder::read_u16(bytes))),
            DataType::I16 => parse_number(2, |bytes| WatchState::I16(ByteOrder::read_i16(bytes))),
            DataType::U32 => parse_number(4, |bytes| WatchState::U32(ByteOrder::read_u32(bytes))),
            DataType::I32 => parse_number(4, |bytes| WatchState::I32(ByteOrder::read_i32(bytes))),
            DataType::U64 => parse_number(8, |bytes| WatchState::U64(ByteOrder::read_u64(bytes))),
            DataType::I64 => parse_number(8, |bytes| WatchState::I64(ByteOrder::read_i64(bytes))),
            DataType::U128 => parse_number(16, |bytes| WatchState::U128(ByteOrder::read_u128(bytes))),
            DataType::I128 => parse_number(16, |bytes| WatchState::I128(ByteOrder::read_i128(bytes))),
            DataType::Char => bytes.get(0).map(|&b| WatchState::Char(b as char)).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
            DataType::NullTerminated(ref element_type) => {
                let mut elements: Vec<WatchState> = Vec::new();

                let bytes_after_null = if let Some(first_null_index) = bytes.iter().position(|&b| b == 0) {
                    let before_and_including_null = &bytes[0..first_null_index + 1];
                    let after_null = &bytes[first_null_index..];

                    let mut left_to_process = before_and_including_null;

                    while left_to_process.len() > 1 { // wait until null empty.
                        let (element, remaining) = element_type.as_watch_state_from_bytes_internal(left_to_process)?;
                        elements.push(element);
                        left_to_process = remaining;
                    }

                    after_null
                } else {
                    &[]
                };

                Ok((WatchState::Array { elements, data_type: *element_type.clone() }, bytes_after_null))
            },
            DataType::HighLowBit => bytes.get(0).cloned().map(|b| WatchState::HighLowBit(if b != 0 { true } else { false })).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
            DataType::IoRegisterStatus => bytes.get(0).cloned().map(WatchState::IoRegisterStatus).ok_or("end of memory".to_string()).map(|s| (s, &bytes[1..])),
        }
    }
}

// Parses a watch from a string. one watch may correspond to multiple backend watches.
fn parse_watch(s: &str) -> Result<Vec<Watch>, String> {
    let s = s.trim();

    fn io_port_from_str(remaining: &str, f: impl FnOnce(char, Option<u8>) -> Vec<Watch>) -> Result<Vec<Watch>, String> {
        let equals_char_index = match remaining.find('=') {
            Some(index) => index,
            None => return Err(format!("expected IO port to include an equals sign and a data type")),
        };
        let specified_port = &remaining[(equals_char_index+1)..];
        let port_letter = match specified_port.chars().nth(0).map(|mut c| { c.make_ascii_uppercase(); c }) {
            Some(c) => c,
            None => return Err(format!("expected IO port to include a port letter and an optional index after the equals sign")),
        };
        let port_index: Option<u8> = match specified_port.chars().nth(1) {
            Some(c) => match c.to_string().parse() {
                Ok(c) => Some(c),
                Err(_) => return Err(format!("port index is not an integer")),
            },
            None => None,
        };

        Ok(f(port_letter, port_index))
    }

    if let Some(remaining) = util::try_consume("datamem", &s) {
        let mut equals_char_indices = remaining.match_indices('=').map(|(i, _)| i);

        let (address, data_type): (String, String) = match (equals_char_indices.next(), equals_char_indices.next()) {
            (None, None) => return Err(format!("expected data memory address to include an address and data type separated by equals signs")),
            (Some(_), None) => return Err(format!("expected data memory address to include a data type separated by equals sign")),
            (Some(a), Some(dt)) => (remaining.chars().skip(a + 1).take(dt - a - 1).collect(), remaining.chars().skip(dt + 1).collect()),
            (None, Some(_)) => unreachable!(),
        };

        address.parse().and_then(|address| data_type.parse().map(|dt| (address, dt))).map(|(address, data_type)| {
            vec![Watch::MemoryAddress { address, data_type, space: MemorySpace::Data }]
        })
    } else if let Some(remaining) = util::try_consume("io-port", &s) {
        io_port_from_str(remaining, |port_letter, port_index| vec![Watch::IoPort { port_letter, port_index }])
    } else if let Some(remaining) = util::try_consume("io-pin", &s) {
        io_port_from_str(remaining, |port_letter, port_index| vec![Watch::IoPin { port_letter, port_index }])
    } else if let Some(remaining) = util::try_consume("io-ddr", &s) {
        io_port_from_str(remaining, |port_letter, port_index| vec![Watch::IoDataDirectionRegister { port_letter, port_index }])
    } else if let Some(remaining) = util::try_consume("io", &s) {
        io_port_from_str(remaining, |port_letter, port_index| vec![
            Watch::IoPort { port_letter, port_index },
            Watch::IoPin { port_letter, port_index },
            Watch::IoDataDirectionRegister { port_letter, port_index },
        ])
    } else if s.chars().filter(|&c| c == '=').count() >= 1 { // symbol name watchables only have one equals sign
        let (symbol_name, data_type_str) = s.split_at(s.find("=").unwrap());
        let data_type_str = &data_type_str[1..];

        data_type_str.parse().map(|data_type| {
            vec![Watch::Symbol { name: symbol_name.to_owned(), data_type }]
        })
    } else {
        Err(format!("invalid WATCHABLE: {}", s))
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
            "u8" => Ok(DataType::U8), "i8" => Ok(DataType::I8),
            "u16" => Ok(DataType::U16), "i16" => Ok(DataType::I16),
            "u32" => Ok(DataType::U32), "i32" => Ok(DataType::I32),
            "u64" => Ok(DataType::U64), "i64" => Ok(DataType::I64),
            "u128" => Ok(DataType::U128), "i128" => Ok(DataType::I128),
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
        assert_eq!(Ok(vec![Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 999, natural_radix: 10 },
            data_type: DataType::U8,
        }]), parse_watch("datamem=999=u8"));

        assert_eq!(Ok(vec![Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 999, natural_radix: 10 },
            data_type: DataType::Char,
        }]), parse_watch("datamem=999=char"));

        assert_eq!(Ok(vec![Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 1, natural_radix: 16 },
            data_type: DataType::NullTerminated(Box::new(DataType::Char)),
        }]), parse_watch("datamem=0x01=null_terminated=char"));

        assert_eq!(Ok(vec![Watch::MemoryAddress {
            space: MemorySpace::Data,
            address: Pointer { address: 0x77, natural_radix: 16 },
            data_type: DataType::I32,
        }]), parse_watch("datamem=0x77=i32"));
    }

    #[test]
    fn can_parse_watchable_symbol() {
        assert_eq!(Ok(vec![Watch::Symbol {
            name: "TEST_BUFFER".to_owned(),
            data_type: DataType::U8,
        }]), parse_watch("TEST_BUFFER=u8"));
    }

    #[test]
    fn can_parse_watchable_io_port() {
        assert_eq!(Ok(vec![Watch::IoPort {
            port_letter: 'A',
            port_index: None,
        }]), parse_watch("io-port=a"));
        assert_eq!(Ok(vec![Watch::IoPort {
            port_letter: 'D',
            port_index: Some(3),
        }]), parse_watch("io-port=D3"));
        assert_eq!(Ok(vec![Watch::IoPin {
            port_letter: 'E',
            port_index: None,
        }]), parse_watch("io-pin=E"));
        assert_eq!(Ok(vec![Watch::IoDataDirectionRegister {
            port_letter: 'A',
            port_index: Some(0),
        }]), parse_watch("io-ddr=A0"));
        assert_eq!(Ok(vec![
            Watch::IoPort { port_letter: 'D', port_index: Some(7) },
            Watch::IoPin { port_letter: 'D', port_index: Some(7) },
            Watch::IoDataDirectionRegister { port_letter: 'D', port_index: Some(7) },
        ]), parse_watch("io=D7"));
    }
}
