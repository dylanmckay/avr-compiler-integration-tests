// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u16

#include <avrlit/boilerplate/unit_test.h>

uint16_t OUTPUT_VALUE = 0xffff;

__attribute__ ((noinline)) uint16_t div(volatile uint16_t a, volatile uint16_t b) {
  return a / b;
}

template<uint16_t divisor>
__attribute__ ((noinline)) uint16_t div_imm(volatile uint16_t a) {
  return a / divisor;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // u16-vs-u16

  // CHECK: changed(OUTPUT_VALUE) = 65531
  OUTPUT_VALUE = div(65531, 1);
  // CHECK: changed(OUTPUT_VALUE) = 32765
  OUTPUT_VALUE = div(65531, 2);
  // CHECK: changed(OUTPUT_VALUE) = 21843
  OUTPUT_VALUE = div(65531, 3);
  // CHECK: changed(OUTPUT_VALUE) = 16382
  OUTPUT_VALUE = div(65531, 4);
  // CHECK: changed(OUTPUT_VALUE) = 8191
  OUTPUT_VALUE = div(65531, 8);
  // CHECK: changed(OUTPUT_VALUE) = 2047
  OUTPUT_VALUE = div(65531, 32);
  // CHECK: changed(OUTPUT_VALUE) = 511
  OUTPUT_VALUE = div(65531, 128);
  // CHECK: changed(OUTPUT_VALUE) = 3
  OUTPUT_VALUE = div(65531, 18323);
  // CHECK: changed(OUTPUT_VALUE) = 12
  OUTPUT_VALUE = div(100, 8);

  // u16-vs-u16 immediate

  // CHECK: changed(OUTPUT_VALUE) = 65531
  OUTPUT_VALUE = div_imm<1>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 32765
  OUTPUT_VALUE = div_imm<2>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 21843
  OUTPUT_VALUE = div_imm<3>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 16382
  OUTPUT_VALUE = div_imm<4>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 8191
  OUTPUT_VALUE = div_imm<8>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 2047
  OUTPUT_VALUE = div_imm<32>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 511
  OUTPUT_VALUE = div_imm<128>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 3
  OUTPUT_VALUE = div_imm<18323>(65531);
  // CHECK: changed(OUTPUT_VALUE) = 12
  OUTPUT_VALUE = div_imm<8>(100);
}

