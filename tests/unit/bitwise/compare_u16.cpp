// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u16

#include <avrlit/boilerplate/unit_test.h>

uint16_t OUTPUT_VALUE = 0xff;

template<int Constant>
__attribute__ ((noinline)) uint16_t compare_with_constant(uint16_t a) {
  return a == Constant;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = compare_with_constant<8>(2432);

  // CHECK: changed(OUTPUT_VALUE) = 1
  OUTPUT_VALUE = compare_with_constant<2432>(2432);

  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = compare_with_constant<16000>(12);

  // CHECK: changed(OUTPUT_VALUE) = 1
  OUTPUT_VALUE = compare_with_constant<-31321>(-31321);
}

