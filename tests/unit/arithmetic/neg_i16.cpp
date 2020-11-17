// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=i16

#include <avrlit/boilerplate/unit_test.h>

int16_t OUTPUT_VALUE = 0xffff;

__attribute__ ((noinline)) int16_t neg(volatile int16_t a) {
  return -a;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // CHECK: changed(OUTPUT_VALUE) = -127
  OUTPUT_VALUE = neg(127);
  // CHECK: changed(OUTPUT_VALUE) = 100
  OUTPUT_VALUE = neg(-100);
  // CHECK: changed(OUTPUT_VALUE) = 255
  OUTPUT_VALUE = neg(-255);
  // CHECK: changed(OUTPUT_VALUE) = -12345
  OUTPUT_VALUE = neg(12345);
  // CHECK: changed(OUTPUT_VALUE) = -31034
  OUTPUT_VALUE = neg(31034);
}


