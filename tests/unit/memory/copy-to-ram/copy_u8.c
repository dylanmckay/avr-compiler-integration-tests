// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u8

#include <avrlit/boilerplate/unit_test.h>

uint8_t OUTPUT_VALUE = 77;

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(OUTPUT_VALUE) = 0

// This next check ensures that the startup routines correctly
// initialize RAM variables.
//
// CHECK: changed(OUTPUT_VALUE) = 77

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = 226
void unit_test(void) {
  OUTPUT_VALUE = 226;
}

