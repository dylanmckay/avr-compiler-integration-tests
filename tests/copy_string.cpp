// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @first_tempfile -O2 && avr-sim @first_tempfile --print-after=datamem=0x123=null_terminated=char

#include <string.h>
#include <avr/sleep.h>

typedef struct {
  char text[200];
} Result;

// CHECK: Hello world
static Result *RESULT = (Result*) 0x123;

// Tells avr-sim to stop running the program.
void sleep_indefinitely(void) {
  asm("cli");
  sleep_enable();
  sleep_bod_disable();

  while(true) {
    sleep_cpu();
  }
}

int main(void) {
  strcpy(RESULT->text, "Hello world, this is a random string!");
  sleep_indefinitely();
  return 0;
}

