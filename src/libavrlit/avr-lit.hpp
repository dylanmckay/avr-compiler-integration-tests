#define F_CPU 16000000UL

#include <stdio.h>
#include <stdint.h>

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
/// Prints a stringified expression and its value.
#define eval(expr) \
  { test::printf("%s = ", #expr); \
    test::println(expr); \
  }

  // Used for vardic generics, look for 'expansion marker'.
  // https://en.wikipedia.org/wiki/Variadic_template
  struct pass {
    template<typename ...T> pass(T...) {}
  };

  /// Prints a format string to the test harness.
  template<typename... Params>
  void printf(const char *fmt, Params... parameters) {
    char buffer[256];
    sprintf(buffer, fmt, parameters...);
    uart::send(buffer);
  }

  void print(unsigned char i) { printf("%hhu", i); }
  void print(signed char i) { printf("%hhi", i); }
  void print(unsigned short i) { printf("%hu", i); }
  void print(signed short i) { printf("%hi", i); }
  void print(unsigned int i) { printf("%u", i); }
  void print(signed int i) { printf("%i", i); }
  void print(unsigned long i) { printf("%lu", i); }
  void print(signed long i) { printf("%li", i); }
  void print(unsigned long long i) { printf("%lu", i); }
  void print(signed long long i) { printf("%li", i); }

  void print(bool b) { b ? print("true") : print("false"); }
  void print(char c) { printf("%c", c); }

  void print(float f) { printf("%f", f); }
  void print(double d) { printf("%f", d); }
  void print(long double d) { printf("%Lf", d); }

  void print(const char *s) { printf("%s", s); }

  /// Prints a list of values.
  template<typename... Params>
  void print(Params... arguments) {
    pass{(print(arguments), 1)...};
  }

  /// Prints a new line character.
  void println() {
    uart::send('\n');
  }

  /// Prints one or more values and then a new line character.
  template<typename... Params>
  void println(Params... arguments) {
    print(arguments...);
    println();
  }

  /// Print a format string to the test harness with a new line.
  template<typename... Params>
  void printlnf(const char *fmt, Params... arguments) {
    printf(fmt, arguments...);
    println();
  }

  /// Calls a function with an optional list of arguments, printing the given
  /// function name, arguments, and the result.
  template<typename Fn, typename... Params>
  void print_call(const char *name, Fn fn, Params... arguments) {
    print(name, "(");
    print(arguments...);

    auto value = fn(arguments...);
    println(") = ", value);
  }

/// Calls a function with an optional list of arguments, printing the function
/// name, arguments, and the result.
#define call(fn, ...) test::print_call(#fn, fn, ##__VA_ARGS__)

  /// Prints an error message and triggers the debugger.
  template<typename... Params>
  void error(const char *fmt, Params... parameters) {
    print("error: ");
    printlnf(fmt, parameters...);

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

