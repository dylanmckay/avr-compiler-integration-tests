// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u16

#include <avrlit/boilerplate/unit_test.h>

uint16_t OUTPUT_VALUE = 0xff;

template<typename A, typename B>
__attribute__ ((noinline)) uint16_t shift_left(volatile A a, volatile B b) {
  return a << b;
}

template<typename A, typename B>
__attribute__ ((noinline)) uint16_t shift_right(volatile A a, volatile B b) {
  return a >> b;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // u16-vs-u16

  // Shift left:
  // CHECK: changed(OUTPUT_VALUE) = 2
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 0);
  // CHECK: changed(OUTPUT_VALUE) = 4
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 1);
  // CHECK: changed(OUTPUT_VALUE) = 8
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 2);
  // CHECK: changed(OUTPUT_VALUE) = 16
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 3);
  // CHECK: changed(OUTPUT_VALUE) = 32
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 4);
  // CHECK: changed(OUTPUT_VALUE) = 64
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 5);
  // CHECK: changed(OUTPUT_VALUE) = 128
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 6);
  // CHECK: changed(OUTPUT_VALUE) = 256
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 7);
  // CHECK: changed(OUTPUT_VALUE) = 512
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 8);
  // CHECK: changed(OUTPUT_VALUE) = 1024
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 9);
  // CHECK: changed(OUTPUT_VALUE) = 2048
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 10);
  // CHECK: changed(OUTPUT_VALUE) = 4096
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 11);
  // CHECK: changed(OUTPUT_VALUE) = 8192
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 12);
  // CHECK: changed(OUTPUT_VALUE) = 16384
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 13);
  // CHECK: changed(OUTPUT_VALUE) = 32768
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 14);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(2, 15);
  // CHECK: changed(OUTPUT_VALUE) = 2040
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(255, 3);
  // CHECK: changed(OUTPUT_VALUE) = 4
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(1, 2);
  // CHECK: changed(OUTPUT_VALUE) = 65280
  OUTPUT_VALUE = shift_left<uint16_t, uint16_t>(0xffff, 8);

  // Shift right:
  // CHECK: changed(OUTPUT_VALUE) = 65535
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 0);
  // CHECK: changed(OUTPUT_VALUE) = 32767
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 1);
  // CHECK: changed(OUTPUT_VALUE) = 16383
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 2);
  // CHECK: changed(OUTPUT_VALUE) = 8191
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 3);
  // CHECK: changed(OUTPUT_VALUE) = 4095
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 4);
  // CHECK: changed(OUTPUT_VALUE) = 2047
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 5);
  // CHECK: changed(OUTPUT_VALUE) = 1023
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 6);
  // CHECK: changed(OUTPUT_VALUE) = 511
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 7);
  // CHECK: changed(OUTPUT_VALUE) = 255
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 8);
  // CHECK: changed(OUTPUT_VALUE) = 127
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 9);
  // CHECK: changed(OUTPUT_VALUE) = 63
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 10);
  // CHECK: changed(OUTPUT_VALUE) = 31
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 11);
  // CHECK: changed(OUTPUT_VALUE) = 15
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 12);
  // CHECK: changed(OUTPUT_VALUE) = 7
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 13);
  // CHECK: changed(OUTPUT_VALUE) = 3
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 14);
  // CHECK: changed(OUTPUT_VALUE) = 1
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 15); // shift everything out but one
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 16);
  // CHECK: changed(OUTPUT_VALUE) = 511
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(65535, 7);
  // CHECK: changed(OUTPUT_VALUE) = 132
  OUTPUT_VALUE = shift_right<uint16_t, uint16_t>(4235, 5);
}

