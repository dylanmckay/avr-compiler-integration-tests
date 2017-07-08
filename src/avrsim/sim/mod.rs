pub use self::firmware::Firmware;

pub mod uart;
pub mod ioctl;
mod firmware;

use simavr;

use std::os::raw::c_int;
use std::ffi::{CString, CStr};
use std::mem;
use std::ptr;

/// An AVR instance.
pub struct Avr {
    avr: *mut simavr::avr_t,
    is_initialised: bool,
}

/// The state of an AVR mcu.
/// Callbacks will update this structure.
#[derive(Debug, PartialEq, Eq)]
pub struct McuState {
    /// Whether we have done the very first initial reset.
    powered_on: bool,
    /// The number of times the AVR has reset.
    reset_count: u64,
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
        let mut avr = Avr { avr: avr, is_initialised: false };

        // avr.raw_mut().set_log(simavr::LOG_WARNING as _);

        let mcu_state = Box::new(McuState::default());
        avr.raw_mut().data = Box::into_raw(mcu_state) as *mut u8;
        avr.raw_mut().reset = Some(util::on_reset);

        avr.set_frequency(16_000_000);
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

    /// Terminates the mcu.
    pub fn terminate(&mut self) {
        unsafe {
            simavr::avr_terminate(self.avr)
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
            if !self.is_initialised {
                simavr::avr_init(self.avr);
                self.is_initialised = true;
            }

            simavr::avr_run(self.avr)
        }.into()
    }

    /// Gets the state of the microcontroller.
    pub fn state(&self) -> &McuState {
        unsafe { util::mcu_state(self.avr) }
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

impl McuState {
    /// Whether the microcontroller has been reset *after* it was first started.
    ///
    /// This will ignore the initial reset signal on startup, only considering
    /// resets after startup.
    fn has_reset(&self) -> bool { self.reset_count > 0 }
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

impl Drop for Avr {
    fn drop(&mut self) {
        let mcu_state: Box<McuState> = unsafe {
            Box::from_raw(self.raw().data as *mut McuState)
        };

        drop(mcu_state)
    }
}

impl Default for McuState {
    fn default() -> McuState {
        McuState {
            powered_on: false,
            reset_count: 0,
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

mod util {
    use super::*;

    /// Hook that runs when the mcu receives a reset signal.
    pub unsafe extern fn on_reset(avr: *mut simavr::avr_t) {
        let mcu_state = self::mcu_state(avr);

        // Check if this is the very first initial reset signal on startup.
        if !mcu_state.powered_on {
            mcu_state.powered_on = true;
        } else {
            // A standard reset.
            mcu_state.reset_count += 1;
        }
    }

    /// Gets the `McuState` that is stored inside the custom data field on an `avr_t`.
    pub unsafe fn mcu_state<'a>(avr: *mut simavr::avr_t) -> &'a mut McuState {
        let ptr = (*avr).data;
        println!("found state at {}", ptr as usize);
        let mut boxed: Box<McuState> = Box::from_raw(ptr as *mut McuState);

        let state: &'a mut McuState = mem::transmute(boxed.as_mut());

        // Forget the box without running the destructors.
        // We only needed to build the box in order to get the underlying
        // reference. The box itself will be freed upon destruction of
        // the `Avr` object.
        mem::forget(boxed);
        state
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn atmega328() -> Avr {
        Avr::with_name("atmega328").unwrap()
    }

    #[test]
    fn can_create_mcu() {
        let avr = atmega328();
        assert_eq!(avr.name(), "atmega328");
    }

    #[test]
    fn new_mcu_state_is_default() {
        let avr = atmega328();
        assert_eq!(avr.state(), &McuState::default());
    }

    #[test]
    fn first_initialise_increments_reset_count() {
        let mut avr = atmega328();

        assert_eq!(avr.state().powered_on, false);
        assert_eq!(avr.state().reset_count, 0);
        avr.run_cycle();
        assert_eq!(avr.state().powered_on, true);
        assert_eq!(avr.state().reset_count, 0);
    }

    #[test]
    fn explicit_resets_after_first_increment_reset_count() {
        let mut avr = atmega328();

        assert_eq!(avr.state().powered_on, false);
        assert_eq!(avr.state().reset_count, 0);

        // Run a few cycles to for good measure.
        for _ in 0..4 {
            avr.run_cycle();
            assert_eq!(avr.state().powered_on, true);
            assert_eq!(avr.state().reset_count, 0);
        }

        avr.reset();
        assert_eq!(avr.state().powered_on, true);
        assert_eq!(avr.state().reset_count, 1);
        avr.reset();
        assert_eq!(avr.state().powered_on, true);
        assert_eq!(avr.state().reset_count, 2);
    }

    #[test]
    fn has_reset_makes_sense() {
        let mut avr = atmega328();
        assert_eq!(avr.state().has_reset(), false);

        // Run a few cycles for good measure.
        for _ in 0..4 {
            avr.run_cycle();
            assert_eq!(avr.state().has_reset(), false);
        }

        avr.reset();
        assert_eq!(avr.state().has_reset(), true);
    }
}
