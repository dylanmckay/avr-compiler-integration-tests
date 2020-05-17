// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=i8

#include <avrlit/boilerplate/unit_test.h>

int8_t OUTPUT_VALUE = 11;

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(OUTPUT_VALUE) = 0

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = -2
void unit_test(void) {
  OUTPUT_VALUE = 0xfe;
}

