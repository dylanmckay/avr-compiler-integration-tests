// RUN: @cxx @cxxflags -mmcu=atmega328p @file -o @first_tempfile -O0 && avr-sim @first_tempfile -w TEST_BUFFER=null_terminated=char

#include <avrlit/boilerplate/unit_test.h>

char TEST_BUFFER[30] = "initialized from data memory";

// This first check validates the assumption that RAM is zeroed at startup.
//
// CHECK: before_execution(TEST_BUFFER) = ""

// This next check ensures that the startup routines correctly
// initialize RAM variables.
//
// CHECK: changed(TEST_BUFFER) = "initialized from data memory"

// This final check ensures that the strcpy correctly updates
// the destination buffer.
// CHECK: after_execution(TEST_BUFFER) = "Hello there, world!"
void unit_test(void) {
  strcpy(TEST_BUFFER, "Hello there, world!");
}

