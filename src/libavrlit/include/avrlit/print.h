#pragma once

#include "thin_libc.h"

// NOTE: make sure to keep this up to date with the constants in avr-sim's 'avr_print.rs'.
#define SB_FLAG_INITIALIZED     (1<<0)
#define SB_FLAG_READY_FOR_WRITE (1<<1)

uint8_t __AVR_SIM_SEND_BUFFER = 0xff;
uint8_t __AVR_SIM_SEND_BUFFER_FLAGS = SB_FLAG_INITIALIZED | SB_FLAG_READY_FOR_WRITE;


void putc(char c) {
  while (!(__AVR_SIM_SEND_BUFFER_FLAGS & SB_FLAG_READY_FOR_WRITE)) __asm__("nop");

  __AVR_SIM_SEND_BUFFER = c;
  __AVR_SIM_SEND_BUFFER_FLAGS &= ~SB_FLAG_READY_FOR_WRITE;
}

void puts(const char *str) {
  while (char c = *str++) {
    putc(c);
  }
}
