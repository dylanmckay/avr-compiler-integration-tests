#pragma once

#include "io_registers.h"

void sleep_enable(void) {
  *SLEEP_REGISTER |= SLEEP_ENABLE_BIT;
}

void sleep_cpu(void) {
  __asm__ __volatile__ ( "sleep" "\n\t" :: );
}

// Tells avr-sim to stop running the program.
void sleep_indefinitely(void) {
  asm("cli");
  sleep_enable();

  while(true) {
    sleep_cpu();
  }
}

