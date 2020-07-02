//! Routines for allowing the AVR to print to the console.

use crate::{
    MemorySpace, WatchableSymbol,
    read_current_memory_address, read_current_memory_address_mut,
};

mod libavrlit_symbol_names {
    //! Symbol names that are used by the 'libavrlit' library.

    pub const SEND_BUFFER: &'static str = "__AVR_SIM_SEND_BUFFER";
    pub const SEND_BUFFER_FLAGS: &'static str = "__AVR_SIM_SEND_BUFFER_FLAGS";
}

#[derive(Clone, Debug)]
pub struct Config {
    symbol_for_send_buffer: WatchableSymbol,
    symbol_for_send_buffer_flags: WatchableSymbol,
}

impl Config {
    /// Creates a new AVR printing config, if it is possible.
    pub fn new(watchable_symbols: &[WatchableSymbol])
        -> Result<Self, String> {
        let lookup_symbol = |symbol_name: &str| {
            match watchable_symbols.iter().find(|s| s.name == symbol_name) {
                Some(s) => Ok(s.clone()),
                None => Err(format!("the AVR executable does not contain the libavrlit special symbol '{}'", symbol_name)),
            }
        };

        let symbol_for_send_buffer = lookup_symbol(libavrlit_symbol_names::SEND_BUFFER)?;
        let symbol_for_send_buffer_flags = lookup_symbol(libavrlit_symbol_names::SEND_BUFFER_FLAGS)?;

        Ok(Config { symbol_for_send_buffer, symbol_for_send_buffer_flags })
    }

    pub fn consume_character(&self, avr: &simavr::Avr) -> Result<Option<char>, String> {
        self.consume_byte(avr).map(|o| o.map(|b| b as char))
    }

    pub fn consume_byte(&self, avr: &simavr::Avr) -> Result<Option<u8>, String> {
        read_current_memory_address(MemorySpace::Data, self.symbol_for_send_buffer_flags.address, avr)?;

        let current_flags = self.get_current_flags(avr)?;
        let ready_for_read = !current_flags.contains(WriteBufferFlags::READY_FOR_WRITE);

        if current_flags.contains(WriteBufferFlags::INITIALIZED) && ready_for_read {
            let byte_written = self.get_current_write_buffer_value(avr)?;

            // Reset the write buffer so the AVR can output the next byte.
            self.set_current_write_buffer_value(0, avr)?;
            self.set_current_flags(current_flags | WriteBufferFlags::READY_FOR_WRITE, avr)?;

            Ok(Some(byte_written))
        } else {
            Ok(None)
        }
    }

    fn get_current_flags(&self, avr: &simavr::Avr) -> Result<WriteBufferFlags, String> {
        read_current_memory_address(MemorySpace::Data, self.symbol_for_send_buffer_flags.address, avr)?
            .get(0)
            .ok_or_else(|| "the debug write buffer flag has no allocated space".to_owned())
            .and_then(|&b| WriteBufferFlags::from_bits(b).ok_or("the debug write buffer flag variable is corrupted".to_string()))
    }

    fn set_current_flags(&self, flags: WriteBufferFlags, avr: &simavr::Avr) -> Result<(), String> {
        let flag_addr: &mut u8 = read_current_memory_address_mut(MemorySpace::Data, self.symbol_for_send_buffer_flags.address, avr)?
            .get_mut(0).unwrap();
        *flag_addr = flags.bits();
        Ok(())
    }

    fn get_current_write_buffer_value(&self, avr: &simavr::Avr) -> Result<u8, String> {
        read_current_memory_address(MemorySpace::Data, self.symbol_for_send_buffer.address, avr)?
            .get(0)
            .cloned()
            .ok_or_else(|| "the debug write buffer has no allocated space".to_owned())
    }

    fn set_current_write_buffer_value(&self, new_value: u8, avr: &simavr::Avr) -> Result<(), String> {
        let buffer_addr: &mut u8 = read_current_memory_address_mut(MemorySpace::Data, self.symbol_for_send_buffer.address, avr)?
            .get_mut(0).unwrap();
        *buffer_addr = new_value;
        Ok(())
    }
}

bitflags! {
    /// NOTE: make sure to keep this up to date with the constants in avrlit's 'print.h'
    pub struct WriteBufferFlags: u8 {
        /// Set and kept once buffer variables are initialized.
        const INITIALIZED = 0b00000001;
        const READY_FOR_WRITE = 0b00000010;
    }
}
