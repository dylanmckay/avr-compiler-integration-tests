// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=i8

#include <avrlit/boilerplate/unit_test.h>

int8_t OUTPUT_VALUE = 0xff;

template<typename A, typename B>
__attribute__ ((noinline)) int8_t add(volatile A a, volatile B b) {
  return a + b;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // i8-vs-i8

  // CHECK: changed(OUTPUT_VALUE) = 5
  OUTPUT_VALUE = add<int8_t, int8_t>(2, 3);
  // CHECK: changed(OUTPUT_VALUE) = 69
  OUTPUT_VALUE = add<int8_t, int8_t>(60, 9);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = add<int8_t, int8_t>(1, -1);
  // CHECK: changed(OUTPUT_VALUE) = -1
  OUTPUT_VALUE = add<int8_t, int8_t>(~1, 1);

  // i8-vs-u8

  // CHECK: changed(OUTPUT_VALUE) = 5
  OUTPUT_VALUE = add<uint8_t, int8_t>(2, 3);
  // CHECK: changed(OUTPUT_VALUE) = 69
  OUTPUT_VALUE = add<int8_t, uint8_t>(60, 9);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = add<uint8_t, int8_t>(1, -1);
  // CHECK: changed(OUTPUT_VALUE) = -1
  OUTPUT_VALUE = add<int8_t, int8_t>(~1, 1);
}

