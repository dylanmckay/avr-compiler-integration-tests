// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u16

#include <avrlit/avrlit.h>

uint32_t OUTPUT_VALUE = 0xbabe;

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(OUTPUT_VALUE) = 0

// The runtime library should initialize RAM for us.:
//
// CHECK: changed(OUTPUT_VALUE) = 47806

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = 51966
int main(void) {
  OUTPUT_VALUE = 0xcafe;

  sleep_indefinitely();
  return 0;
}

