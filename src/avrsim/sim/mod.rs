pub use self::firmware::Firmware;

pub mod pty;
mod firmware;

use simavr;

use std::ffi::{CString, CStr};
use std::ptr;

/// An AVR instance.
pub struct Avr {
    avr: *mut simavr::avr_t,
}

impl Avr {
    /// Creates a new avr instance.
    pub fn with_name(name: &str) -> Result<Self, &'static str> {
        let name = CString::new(name).unwrap();
        let avr = unsafe { simavr::avr_make_mcu_by_name(name.as_ptr()) };

        if avr == ptr::null_mut() {
            return Err("could not create avr sim");
        }

        unsafe { simavr::avr_init(avr); }

        Ok(Avr {
            avr: avr,
        })
    }

    /// Resets the mcu.
    pub fn reset(&mut self) {
        unsafe {
            simavr::avr_reset(self.avr);
        }
    }

    /// Loads a firmware.
    pub fn load(&mut self, firmware: &Firmware) {
        unsafe {
            simavr::avr_load_firmware(self.avr,
                                      // This parameter is probably missing a 'const' qualifier
                                      firmware.raw() as *const _ as *mut _)
        }
    }

    /// Gets the name of the mcu.
    pub fn name(&self) -> &str {
        let name = unsafe { CStr::from_ptr(self.raw().mmcu) };
        name.to_str().expect("mcu name is not valid utf-8")
    }

    /// Gets the frequency of the mcu.
    pub fn frequency(&self) -> u32 {
        self.raw().frequency
    }

    /// Sets the frequency of the mcu.
    pub fn set_frequency(&mut self, freq: u32) {
        self.raw_mut().frequency = freq;
    }

    /// Gets a reference to the underlying `avr_t` structure.
    pub fn raw(&self) -> &simavr::avr_t { unsafe { &*self.avr } }
    /// Gets a mutable reference to the underlying `avr_t` structure.
    pub fn raw_mut(&mut self) -> &mut simavr::avr_t { unsafe { &mut *self.avr } }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_create_mcu() {
        let avr = Avr::with_name("atmega328").unwrap();
        assert_eq!(avr.name(), "atmega328");
    }
}
