// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u32

#include <avrlit/boilerplate/unit_test.h>

uint32_t OUTPUT_VALUE = 0xabcdef;

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(OUTPUT_VALUE) = 0

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = 3735928559
void unit_test(void) {
  OUTPUT_VALUE = 0xdeadbeef;
}

