use sim::Avr;

use simavr;
use std::os::raw::c_void;

use std::ffi::CString;
use std::ptr;



// #define AVR_IOCTL_DEF	(	 	_a,
//  	_b,
//  	_c,
//  	_d
// )		   (((_a) << 24)|((_b) << 16)|((_c) << 8)|((_d)))

fn ioctl(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | ((d as u32)))
}

fn uart(name: char) -> u32 {
    ioctl('u' as u8, 'a' as u8, 'r' as u8, name as u8)
}

/// The names of the IRQs we want to attach to.
const IRQ_NAMES: &'static [&'static str] = &[
    "8<uart_pty.in", // Must be first
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

        let uart_name = '0';
        let uart = uart(uart_name);

        let src = simavr::avr_io_getirq(avr.raw_mut() as *mut _, uart, simavr::UART_IRQ_OUTPUT as _);
        let dst = simavr::avr_io_getirq(avr.raw_mut() as *mut _, uart, simavr::UART_IRQ_INPUT as _);

        if src != ptr::null_mut() && dst != ptr::null_mut() {
            simavr::avr_connect_irq(src, irq);
        }
    }
}

unsafe extern "C" fn irq_input_hook(_irq: *mut simavr::avr_irq_t,
                                    value: u32,
                                    _param: *mut c_void) {
    println!("received data: '{}'", value);
}
