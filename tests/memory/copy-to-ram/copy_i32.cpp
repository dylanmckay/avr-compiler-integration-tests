// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=i32

#include <avrlit/boilerplate/unit_test.h>

int32_t OUTPUT_VALUE = 0xabcdef;

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(OUTPUT_VALUE) = 0

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = -16843010
void unit_test(void) {
  OUTPUT_VALUE = 0xfefefefe;
}

