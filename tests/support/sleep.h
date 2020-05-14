#pragma once

#include "io_registers.h"

void sleep_enable(void) {
  *SLEEP_REGISTER |= SLEEP_ENABLE_BIT;
}

void sleep_bod_disable(void) {
}

void sleep_cpu(void) {
  __asm__ __volatile__ ( "sleep" "\n\t" :: );
}

