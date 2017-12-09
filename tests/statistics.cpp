// RUN: @cxx -mmcu=atmega328p @file -o /dev/stdout | avr-sim

#include "../src/libavrlit/avr-lit.hpp"

using namespace test;

static const int SMALL_NUMBERS[] = {
  1, 2, 3, 4, 5, 6, 7, 8, 9, 10
};

static const int BIG_NUMBERS[] = {
  100, 200, 300, 400, 500, 600, 700, 800, 900, 1000,
};

int average(const int values[], int count) {
  int sum = 0;
  for (int i=0; i<count; i++) {
    sum += values[i];
  }

  return sum / count;
}

void run_test() {
  // CHECK: average(SMALL_NUMBERS, 10) = 5
  eval(average(SMALL_NUMBERS, 10));
  // CHECK: average(BIG_NUMBERS, 10) = 550
  eval(average(BIG_NUMBERS, 10));
}

