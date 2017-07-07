use simavr;

use std::ffi::CString;
use std::mem;
use std::path::Path;

/// An AVR firmware.
pub struct Firmware {
    /// The underlying firmware representation.
    firmware: simavr::elf_firmware_t,
}

impl Firmware {
    /// Create a firmware from its underlying representation.
    pub fn from_raw(firmware: simavr::elf_firmware_t) -> Self {
        Firmware { firmware: firmware }
    }

    /// Reads firmware from an ELF file on disk.
    pub fn read_elf<P>(path: P) -> Result<Self, ()>
        where P: AsRef<Path> {
        let path = CString::new(path.as_ref().to_str().unwrap()).unwrap();

        let firmware = unsafe {
            let mut firmware = mem::zeroed();

            let result = simavr::elf_read_firmware(path.as_ptr(), &mut firmware);
            assert_eq!(result, 0, "could not read firmware");
            firmware
        };

        Ok(Firmware::from_raw(firmware))
    }

    /// Gets the underlying value of the firmware.
    pub fn raw(&self) -> &simavr::elf_firmware_t { &self.firmware }
    /// Gets the underlying value of the firmware.
    pub fn raw_mut(&mut self) -> &mut simavr::elf_firmware_t { &mut self.firmware }
}

