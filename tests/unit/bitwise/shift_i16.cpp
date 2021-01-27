// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=i16

#include <avrlit/boilerplate/unit_test.h>

int16_t OUTPUT_VALUE = 0xff;

template<int16_t amount>
__attribute__ ((noinline)) int16_t shift_left(volatile int16_t a) {
  return a << amount;
}

template<int16_t amount>
__attribute__ ((noinline)) int16_t shift_right(volatile int16_t a) {
  return a >> amount;
}

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // i16-vs-i16

  // Shift left:
  // CHECK: changed(OUTPUT_VALUE) = 2
  OUTPUT_VALUE = shift_left<0>(2);
  // CHECK: changed(OUTPUT_VALUE) = 4
  OUTPUT_VALUE = shift_left<1>(2);
  // CHECK: changed(OUTPUT_VALUE) = 8
  OUTPUT_VALUE = shift_left<2>(2);
  // CHECK: changed(OUTPUT_VALUE) = 16
  OUTPUT_VALUE = shift_left<3>(2);
  // CHECK: changed(OUTPUT_VALUE) = 32
  OUTPUT_VALUE = shift_left<4>(2);
  // CHECK: changed(OUTPUT_VALUE) = 64
  OUTPUT_VALUE = shift_left<5>(2);
  // CHECK: changed(OUTPUT_VALUE) = 128
  OUTPUT_VALUE = shift_left<6>(2);
  // CHECK: changed(OUTPUT_VALUE) = 256
  OUTPUT_VALUE = shift_left<7>(2);
  // CHECK: changed(OUTPUT_VALUE) = 512
  OUTPUT_VALUE = shift_left<8>(2);
  // CHECK: changed(OUTPUT_VALUE) = 1024
  OUTPUT_VALUE = shift_left<9>(2);
  // CHECK: changed(OUTPUT_VALUE) = 2048
  OUTPUT_VALUE = shift_left<10>(2);
  // CHECK: changed(OUTPUT_VALUE) = 4096
  OUTPUT_VALUE = shift_left<11>(2);
  // CHECK: changed(OUTPUT_VALUE) = 8192
  OUTPUT_VALUE = shift_left<12>(2);
  // CHECK: changed(OUTPUT_VALUE) = 16384
  OUTPUT_VALUE = shift_left<13>(2);
  // CHECK: changed(OUTPUT_VALUE) = -32768
  OUTPUT_VALUE = shift_left<14>(2);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = shift_left<15>(2);
  // CHECK: changed(OUTPUT_VALUE) = 2040
  OUTPUT_VALUE = shift_left<3>(255);
  // CHECK: changed(OUTPUT_VALUE) = 4
  OUTPUT_VALUE = shift_left<2>(1);
  // CHECK: changed(OUTPUT_VALUE) = -256
  OUTPUT_VALUE = shift_left<8>(0xffff);

  // Shift right:
  // CHECK: changed(OUTPUT_VALUE) = 2
  OUTPUT_VALUE = shift_right<0>(2);
  // CHECK: changed(OUTPUT_VALUE) = 1
  OUTPUT_VALUE = shift_right<1>(2);
  // CHECK: changed(OUTPUT_VALUE) = 0
  OUTPUT_VALUE = shift_right<2>(2);
  // CHECK: changed(OUTPUT_VALUE) = -31832
  OUTPUT_VALUE = shift_right<0>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -15916
  OUTPUT_VALUE = shift_right<1>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -7958
  OUTPUT_VALUE = shift_right<2>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -3979
  OUTPUT_VALUE = shift_right<3>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -1990
  OUTPUT_VALUE = shift_right<4>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -995
  OUTPUT_VALUE = shift_right<5>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -498
  OUTPUT_VALUE = shift_right<6>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -249
  OUTPUT_VALUE = shift_right<7>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -125
  OUTPUT_VALUE = shift_right<8>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -63
  OUTPUT_VALUE = shift_right<9>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -32
  OUTPUT_VALUE = shift_right<10>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -16
  OUTPUT_VALUE = shift_right<11>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -8
  OUTPUT_VALUE = shift_right<12>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -4
  OUTPUT_VALUE = shift_right<13>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -2
  OUTPUT_VALUE = shift_right<14>(-31832);
  // CHECK: changed(OUTPUT_VALUE) = -1
  OUTPUT_VALUE = shift_right<15>(-31832);
}

