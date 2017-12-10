#define F_CPU 16000000UL

#include "io/io.h"
#include "interrupt.h"
#include "stdlib/avr/sleep.h"

#include "TinyCStringBuilder/CStringBuilder.hpp"

#define USART_BAUDR0ATE 9600
#define BAUD_PRESCALE (((F_CPU / (USART_BAUDR0ATE * 16UL))) - 1)

/// Assert that a condition is true.
#define assert(expr, message) \
  { test::assert_impl(expr, __FILE__, __LINE__, __FUNCTION__, #expr, message); }

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
    while (!(UCSR0A & _BV(RXC0))) {
      asm ("");
    }
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
  { \
    test::print(#expr); \
    test::print(" = "); \
    test::println(expr); \
  }

  // Used for vardic generics, look for 'expansion marker'.
  // https://en.wikipedia.org/wiki/Variadic_template
  struct pass {
    template<typename ...T> pass(T...) {}
  };

  /// Prints a string.
  void put(const char *s) {
    uart::send(s);
  }

  /// Writes a value to the UART as a string.
  template<typename T>
  void put_value(T val) {
    char buffer[256];
    tcsb::CStringBuilder builder(buffer);

    builder << val;
    put(builder.cstr());
  }

  /// Prints a single value to the UART as a string.
  template<typename T>
  void put(T value);

  template<> void put(int8_t i) { put_value(i); }
  template<> void put(uint16_t i) { put_value(i); }
  template<> void put(int16_t i) { put_value(i); }
  template<> void put(uint32_t i) { put_value(i); }
  template<> void put(int32_t i) { put_value(i); }
  template<> void put(uint64_t i) { put_value(i); }
  template<> void put(int64_t i) { put_value(i); }
  template<> void put(signed int i) { put_value(int16_t(i)); }
  template<> void put(unsigned int i) { put_value(uint16_t(i)); }
  template<> void put(bool b) { b ? put("true") : put("false"); }
  template<> void put(char c) { put_value(c); }
  template<> void put(float f) { put_value(f); }
  template<> void put(double d) { put(float(d)); }

  /// Prints a list of values.
  template<typename... Params>
  void print(Params... arguments) {
    pass{(put(arguments), 1)...};
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
  void error(Params... parameters) {
    print("error: ");
    println(parameters...);

    // Cause some weird memory accesses to trigger
    // an error.
    for (int i=0; ; i++) {
      volatile char* p = ((volatile char*)0x00) + i;
      *p;
    }
  }

  /// Asserts that some condition is true.
  void assert_impl(bool condition,
                   const char *file,
                   uint32_t line,
                   const char *func_name,
                   const char *expr,
                   const char *message) {
    if (!condition) {
      error("assertion failed [", file, ":", func_name, "():", line, "] ", expr,
            "(", message, ") is not true");
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

