#pragma once


// #define IO_REGISTER(addr) addr
// _SFR_IO8(0x33)

typedef volatile uint8_t* IoReg;

// This will not work for all CPUs.
const IoReg SLEEP_REGISTER = (IoReg)0x33;

// On most devices, this bit is 1. Sometimes it is
// 1<<5. Sometimes not. This will not work for all CPUs.
const uint8_t SLEEP_ENABLE_BIT = 1<<1;

