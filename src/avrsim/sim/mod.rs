pub use self::firmware::Firmware;

pub mod uart;
pub mod ioctl;
mod firmware;

use simavr;

use std::os::raw::c_int;
use std::ffi::{CString, CStr};
use std::ptr;

/// An AVR instance.
pub struct Avr {
    avr: *mut simavr::avr_t,
}

/// The state of a simulated AVR.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// Before initialization is finished.
    Limbo = 0,
    /// All is stopped, timers included.
    Stopped,
    /// Running freely.
    Running,
    /// We're sleeping until an interrupt.
    Sleeping,
    /// Run ONE instruction.
    Step,
    /// Tell gdb it's all OK, and give it registers.
    StepDone,
    /// AVR simulation stopped gracefully.
    Done,
    /// AVR simulation crashed (watchdog fired).
    Crashed,
}

impl Avr {
    pub unsafe fn from_raw(avr: *mut simavr::avr_t) -> Self {

        let mut avr = Avr { avr: avr };
        avr.set_frequency(16_000_000);
        // Enable trace.
        // avr.raw_mut().set_log(simavr::LOG_WARNING as _);

        simavr::avr_init(avr.avr);
        avr
    }

    /// Creates a new avr instance.
    pub fn with_name(name: &str) -> Result<Self, &'static str> {
        let name = CString::new(name).unwrap();
        let avr = unsafe { simavr::avr_make_mcu_by_name(name.as_ptr()) };

        if avr == ptr::null_mut() {
            return Err("could not create avr sim");
        }

        Ok(unsafe { Avr::from_raw(avr) })
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

    /// Runs a single cycle.
    pub fn run_cycle(&mut self) -> State {
        unsafe {
            simavr::avr_run(self.avr)
        }.into()
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

    pub unsafe fn underlying(&mut self) -> *mut simavr::avr_t {
        self.avr
    }

    /// Gets a reference to the underlying `avr_t` structure.
    pub fn raw(&self) -> &simavr::avr_t { unsafe { &*self.avr } }
    /// Gets a mutable reference to the underlying `avr_t` structure.
    pub fn raw_mut(&mut self) -> &mut simavr::avr_t { unsafe { &mut *self.avr } }
}

impl State {
    /// Checks if the state represents a running simulation, regardless
    /// of success of failure.
    pub fn is_running(&self) -> bool {
        match *self {
            State::Limbo => true,
            State::Stopped => false,
            State::Running => true,
            State::Sleeping => true,
            State::Step => true,
            State::StepDone => true,
            State::Done => false,
            State::Crashed => false,
        }
    }
}

impl From<c_int> for State {
    fn from(v: c_int) -> Self {
        match v {
            0 => State::Limbo,
            1 => State::Stopped,
            2 => State::Running,
            3 => State::Sleeping,
            4 => State::Step,
            5 => State::StepDone,
            6 => State::Done,
            7 => State::Crashed,
            _ => panic!("unknown state discriminator: {}", v),
        }
    }
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
