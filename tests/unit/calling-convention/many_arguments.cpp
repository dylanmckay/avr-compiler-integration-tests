// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUTPUT_VALUE=u16

// This test case is motivated by two patches
//
//   - https://reviews.llvm.org/D78579
//   - https://reviews.llvm.org/D78581
//
// It ensures that AVR correctly pushes arguments onto the stack.

#include <avrlit/boilerplate/unit_test.h>

uint16_t OUTPUT_VALUE = 0xbabe;

volatile int8_t ARBITRARY = 12;

__attribute__ ((noinline)) int16_t callStuff(uint64_t a, uint64_t b, uint16_t c, uint16_t d) {
  return d;
}

// This first check validates the assumption that RAM is zeroed at startup.
//

// This final check ensures that the assignment correctly updates the global variable.
// CHECK: after_execution(OUTPUT_VALUE) = 4
void unit_test(void) {
  for (int i=0; i<10; ++i) {
    OUTPUT_VALUE = callStuff(ARBITRARY,2,4,4);
  }
}

