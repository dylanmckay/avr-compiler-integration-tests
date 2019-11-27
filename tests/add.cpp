// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @first_tempfile -O2 && avr-sim @first_tempfile

#include "../src/libavrlit/avr-lit.hpp"

using namespace test;

void run_test() {
  // CHECK: 1 + 1 = 2
  eval(1 + 1);
  test::put("hello");
}

