// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=i16

#include <avrlit/boilerplate/unit_test.h>

int16_t OUTPUT_VALUE = 0xbabe;

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(OUTPUT_VALUE) = 0

// The runtime library should initialize RAM for us:
//
// CHECK: changed(OUTPUT_VALUE) = -17730

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = -20561
void unit_test(void) {
  OUTPUT_VALUE = 0xafaf;
}

