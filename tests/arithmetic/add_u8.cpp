// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u8

#include <avrlit/boilerplate/unit_test.h>

uint8_t OUTPUT_VALUE = 0xff;

template<typename A, typename B>
__attribute__ ((noinline)) uint8_t add(volatile A a, volatile B b) {
  return a + b;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // u8-vs-u8

  // CHECK: changed(OUTPUT_VALUE) = 5
  OUTPUT_VALUE = add<uint8_t, uint8_t>(2, 3);
  // CHECK: changed(OUTPUT_VALUE) = 69
  OUTPUT_VALUE = add<uint8_t, uint8_t>(60, 9);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = add<uint8_t, uint8_t>(1, -1);
  // CHECK: changed(OUTPUT_VALUE) = 255
  OUTPUT_VALUE = add<uint8_t, uint8_t>(~1, 1);

  // u8-vs-i8

  // CHECK: changed(OUTPUT_VALUE) = 5
  OUTPUT_VALUE = add<uint8_t, int8_t>(2, 3);
  // CHECK: changed(OUTPUT_VALUE) = 69
  OUTPUT_VALUE = add<int8_t, uint8_t>(60, 9);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = add<uint8_t, int8_t>(1, -1);
  // CHECK: changed(OUTPUT_VALUE) = 50
  OUTPUT_VALUE = add<int8_t, int8_t>(-50, 100);
}

