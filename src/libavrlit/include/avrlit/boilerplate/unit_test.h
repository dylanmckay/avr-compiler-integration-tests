#pragma once

#include "../avrlit.h"

// Forward-declar the unit test entry point.
void unit_test();

int main(void) {
  // Run the unit test entry point.
  unit_test();

  // Sleeping forever instructs the simulator to stop processing.
  sleep_indefinitely();
  return 0;
}
