// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @first_tempfile -O0 && avr-sim @first_tempfile --print-after=TEST_BUFFER=null_terminated=char

#include "support/support.h"

// char baef[100] = { 13};
// int fart[8] = { ~1};
char TEST_BUFFER[30] = "uninit";

// #include <avr/sleep.h>
//
// typedef struct {
//   char text[25];
// } Result;
//
// // CHECK: Hello world
// static Result *RESULT = (Result*) 0x60;

// Tells avr-sim to stop running the program.
void sleep_indefinitely(void) {
  asm("cli");
  sleep_enable();
  // sleep_bod_disable();

  while(true) {
    sleep_cpu();
  }
}

int main(void) {
  // __asm__("nop");
  // char * foo = (char*) 0x50;
  // char foo[100];
  // *foo = 'a';
  // for(unsigned i=0; i<16000; i++) __asm__("nop");
  // TEST_BUFFER[0] = 'F';
  // TEST_BUFFER[1] = 0;
  // memcpymate(TEST_BUFFER, "Hello there, world!", 6);
  // strcpyz(TEST_BUFFER, "H");
  sleep_indefinitely();
  return 0;
}

