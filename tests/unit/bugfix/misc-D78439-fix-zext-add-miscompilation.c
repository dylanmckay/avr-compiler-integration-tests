// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile -w TEST_STATE=u32

// This integration test validates AVR compilation of a zero extended addition.
//
// Before D78439, the add function would incorrectly compute '393216', which should've
// actually been '524288'.
//
// Ayke(@aykevl) says it best in a comment on D78439:
//
//     Adding 0x7ffff and 1 should result in 0x80000. However, without this patch it results in 0x60000.
//
//     Assembly without this patch:
//
//     d20:       64 0f           add     r22, r20
//     d22:       75 1f           adc     r23, r21
//     d24:       80 40           sbci    r24, 0x00       ; 0
//     d26:       90 40           sbci    r25, 0x00       ; 0
//     d28:       08 95           ret
//
//     With this patch:
//
//     d1c:       20 e0           ldi     r18, 0x00       ; 0
//     d1e:       30 e0           ldi     r19, 0x00       ; 0
//     d20:       64 0f           add     r22, r20
//     d22:       75 1f           adc     r23, r21
//     d24:       82 1f           adc     r24, r18
//     d26:       93 1f           adc     r25, r19
//     d28:       08 95           ret
//
//     The sbci not only subtracts an immediate (in this case zero), but it also subtracts the carry bit.
//     Therefore it definitely does have an effect. This also is true for the last two adc instructions: they
//     are needed to add the carry bit in case the previous add instruction caused a wraparound. However sbci
//     and adc use the carry in the opposite direction (subtracting or adding it to their result).


#include <avrlit/boilerplate/unit_test.h>

uint32_t TEST_STATE = 0x1;

// in one file
__attribute__ ((noinline)) uint32_t add(uint32_t a, uint16_t b) {
    return a + b;
}

// in another file
__attribute__ ((noinline)) uint32_t add(uint32_t a, uint16_t b);

void unit_test(void) {
  // CHECK: after_execution(TEST_STATE) = 524288
  TEST_STATE = add(0x7ffff, 1);
}

