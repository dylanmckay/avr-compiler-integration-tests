// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile

#include <avrlit/boilerplate/unit_test.h>

int8_t OUTPUT_VALUE = 11;

// CHECK: hi, from the AVR!

void unit_test(void) {
  puts("hi, from the AVR!\n");
}

