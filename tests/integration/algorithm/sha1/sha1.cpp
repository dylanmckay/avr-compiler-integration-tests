// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @tempfile -O0 && avr-sim @tempfile --print-after OUTPUT_VALUE=u32

#include <avrlit/boilerplate/unit_test.h>
#include "sha1.hpp"

uint32_t OUTPUT_VALUE = 0xff;

typedef struct {
  uint32_t bytes[5];
} digest_t;

digest_t performSHA1(const uint8_t* bytes, uint32_t size) {
  sha1::SHA1 s;
  s.processBytes(bytes, size);
  digest_t digest = {0};
  s.getDigest(digest.bytes);
  return { digest };
}



// CHECK: OUTPUT_VALUE = 2868168221
void unit_test(void) {
  const char *input = "hello";

  digest_t digest = performSHA1((uint8_t*)input, strlen(input));

  OUTPUT_VALUE = digest.bytes[0];
}

