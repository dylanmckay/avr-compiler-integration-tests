use sim::Avr;

use simavr;
use std::os::raw::c_void;

use std::ffi::CString;
use std::ptr;

/// The names of the IRQs we want to attach to.
const IRQ_NAMES: &'static [&'static str] = &[
    "8<uart_pty.in",
    "8>uart_pty.out",
];

pub struct Pty {
}

/// Attaches.
pub fn attach(avr: &mut Avr) {
    let irq_names: Vec<_> = IRQ_NAMES.iter()
                                      .map(|&irq| CString::new(irq).unwrap())
                                      .collect();

    let mut irq_names: Vec<_> = irq_names.iter().map(|irq| irq.as_ptr()).collect();


    unsafe {
        let irq = simavr::avr_alloc_irq(&mut avr.raw_mut().irq_pool, 0,
            irq_names.len() as u32, irq_names.as_mut_ptr());

        let param = ptr::null_mut();
        simavr::avr_irq_register_notify(irq, Some(self::irq_input_hook), param);
    }
}

unsafe extern "C" fn irq_input_hook(_irq: *mut simavr::avr_irq_t,
                                    value: u32,
                                    _param: *mut c_void) {
    println!("received data: '{}'", value);
}
