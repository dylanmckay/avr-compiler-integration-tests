#define F_CPU 16000000UL

#include <stdio.h>

#include <avr/io.h>
#include <avr/interrupt.h>
#include <avr/sleep.h>

#define USART_BAUDR0ATE 9600
#define BAUD_PRESCALE (((F_CPU / (USART_BAUDR0ATE * 16UL))) - 1)

/// Assert that a condition is true.
#define ASSERT(expr, message) \
  { test::assert(expr, __FILE__, __LINE__, __FUNCTION__, #expr, message); }

namespace uart {
  volatile unsigned char value;

  void init(void){
    // Set baud rate.
    UBRR0L = BAUD_PRESCALE;
    UBRR0H = (BAUD_PRESCALE >> 8);

    // Enable transmission and reception.
    UCSR0B |= _BV(RXEN0) | _BV(TXEN0);
    // Set data format.
    UCSR0C = ((1<<UCSZ00)|(1<<UCSZ01));
  }

  /// Wait for a byte to be received and return it.
  uint8_t receive() {
    while (!(UCSR0A & _BV(RXC0))) ;
      return (uint8_t) UDR0;
  }


  /// Sends a single byte through the UART.
  void send(uint8_t data){
    while(!(UCSR0A & (1<<UDRE0)));
    // Transmit data
    UDR0 = data;
  }

  /// Sends a string through the uart.
  void send(const char *str) {
    while (*str) {
      send(*str++);
    }
  }
}

namespace power {
  /// Put the AVR into sleep indefinitely.
  void sleep_indefinitely() {
    // Disable interrupts.
    cli();

    /// Get ready to sleep.
    sleep_enable();
    /// Disable the brown-out detector.
    sleep_bod_disable();

    // We place this in a loop just in case cosmic rays mess with
    // our memory,
    // We can never ever actually return from the first sleep_cpu()
    // because interrupts are disabled so there's nothing to wake
    // us up.
    while (true) {
      /// Sleep the cpu.
      sleep_cpu();
    }
  }
}

namespace test {
  /// Prints a format string to the test harness.
  template<typename... Params>
  void print(const char *fmt, Params... parameters) {
    char buffer[256];
    sprintf(buffer, fmt, parameters...);
    uart::send(buffer);
  }

  /// Print a format string to the test harness with a new line.
  template<typename... Params>
  void println(const char *fmt, Params... parameters) {
    print(fmt, parameters...);
    uart::send('\n');
  }

  /// Prints an error message and triggers the debugger.
  template<typename... Params>
  void error(const char *fmt, Params... parameters) {
    print("error: ");
    println(fmt, parameters...);

    // Cause some weird memory accesses to trigger
    // an error.
    for (int i=0; ; i++) {
      volatile char* p = ((volatile char*)0x00) + i;
      *p;
    }
  }

  /// Asserts that some condition is true.
  void assert(bool condition,
              const char *file,
              uint32_t line,
              const char *func_name,
              const char *expr,
              const char *message) {
    if (!condition) {
      error("assertion failed [%s:%s():%d] %s (%s) is not true",
            file, func_name, line, expr, message);
    }
  }
}

/// The test entry point.
void run_test();

int main() {
  uart::init();
  sei();

  run_test();
  power::sleep_indefinitely();
}

