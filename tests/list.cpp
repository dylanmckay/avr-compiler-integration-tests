// RUN: @cxx -mmcu=atmega328p @file -o /dev/stdout | avr-sim

#include "../src/libavrlit/avr-lit.hpp"
#include "list.hpp"

using namespace test;

void run_test() {
  List<int> list;
  assert(list.size() == 0, "new list should be empty");
  list.add(3);
  assert(list.size() == 1, "list should have one item");
}

