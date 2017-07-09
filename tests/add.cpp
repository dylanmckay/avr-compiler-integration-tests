// RUN: avr-gcc -mmcu=atmega328p @file -o /dev/stdout | avr-sim

#include "../src/libavrlit/avr-lit.hpp"

using namespace test;

// CHECK: eval(1 + 1) = 2
void run_test() {
  eval(1 + 1);
}

