#ifndef ALLOY_TYPES_H
#define ALLOY_TYPES_H

// Standard integer types
typedef unsigned char uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int uint32_t;
typedef unsigned long long uint64_t;

typedef signed char int8_t;
typedef signed short int16_t;
typedef signed int int32_t;
typedef signed long long int64_t;

#ifdef ARCH_I686
typedef uint32_t size_t;
typedef int32_t ssize_t;
typedef uint32_t uintptr_t;
typedef int32_t intptr_t;
#else
typedef uint64_t size_t;
typedef int64_t ssize_t;
typedef uint64_t uintptr_t;
typedef int64_t intptr_t;
#endif

// NULL pointer
#define NULL ((void*)0)

// Boolean type for C (C++ has built-in bool)
#ifndef __cplusplus
typedef _Bool bool;
#define true 1
#define false 0
#endif

#endif // ALLOY_TYPES_H
