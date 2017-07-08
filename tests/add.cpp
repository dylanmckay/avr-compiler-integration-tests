// RUN: avr-gcc -mmcu=atmega328p  @file -o /tmp/add.elf && avrsim /tmp/add.elf

#include "avrlit.hpp"

using namespace test;

// CHECK: bar
// CHECK: foo
void run_test() {
  int a = 1 + 1;

  println("bar");
  println("foo");
  // error("your mum");
}

