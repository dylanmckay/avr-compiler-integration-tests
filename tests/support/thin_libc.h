#pragma once

/**
 * A thin, freestanding header-only basically-empty libc implementation for AVR.
 */

typedef unsigned char uint8_t;
typedef signed char int8_t;
typedef unsigned int uint16_t;
typedef signed int int16_t;
typedef unsigned long int uint32_t;
typedef signed long int int32_t;
typedef unsigned long long int uint64_t;
typedef signed long long int int64_t;

typedef uint16_t size_t;

void* memcpymate(void *dest, const void* src, size_t n) {
  uint8_t *destPtr = (uint8_t*)dest;
  uint8_t *srcPtr = (uint8_t*)src;

  for (size_t i=0; i<n; ++i) {
    (destPtr[i]) = (srcPtr[i]);
  }

  return dest;
}

// void* memcpy(void *dest, const void* src, size_t n) {
//   return (void*)0;
// }

// char* strcpyz(char* dest, const char* src) {
//   char *save = dest;
//   while(( dest[0] = src[0]) != '0') {
//     dest++;
//     src++;
//   }
//   // while(*dest++ = *src++);
//   return save;
// }


