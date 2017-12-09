// RUN: @cxx -mmcu=atmega328p @file -o /dev/stdout | avr-sim

#include "../src/libavrlit/avr-lit.hpp"

using namespace test;

void run_test() {
// CHECK: 1 + 1 = 2
  eval(1 + 1);
// CHECK: 5 + 5 = 10
  eval(5 + 5);
// CHECK: 0 + -1 = -1
  eval(0 + -1);
}

