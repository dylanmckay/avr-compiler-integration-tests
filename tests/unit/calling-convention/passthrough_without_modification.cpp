// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w OUT_U64_A=u64 -w OUT_I64_B=i64 -w OUT_U16_C=u16 -w OUT_U16_D=u16 -w OUT_U8_E=u8

#include <avrlit/boilerplate/unit_test.h>

volatile uint64_t OUT_U64_A = 12;
volatile uint64_t OUT_I64_B = 12;
volatile uint64_t OUT_U16_C = 12;
volatile uint64_t OUT_U16_D = 12;
volatile uint64_t OUT_U8_E = 127;

uint64_t ARBITRARY_U64 = 42;

__attribute__ ((noinline)) uint16_t callStuff(uint64_t a, int64_t b, uint16_t c, uint16_t d, uint8_t e) {
  OUT_U64_A = a;
  OUT_I64_B = b;
  OUT_U16_C = c;
  OUT_U16_D = d;
  OUT_U8_E = e;

  return d;
}

// This first check validates the assumption that RAM is zeroed at startup.
//

// This final check ensures that the assignment correctly updates the global variable.
void unit_test(void) {
  // CHECK: changed(OUT_U64_A) = 42
  // CHECK: changed(OUT_I64_B) = -1844674407370
  // CHECK: changed(OUT_U16_C) = 4
  // CHECK: changed(OUT_U16_D) = 4
  // CHECK: changed(OUT_U8_E) = 255
  callStuff(ARBITRARY_U64,-1844674407370,4,4, ~0);
}

