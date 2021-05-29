#pragma once

typedef unsigned char      u8;
typedef unsigned short     u16;
typedef unsigned int       u32;
typedef unsigned long long u64;

typedef signed char      s8;
typedef signed short     s16;
typedef signed int       s32;
typedef signed long long s64;

typedef float  f32;
typedef double f64;

typedef u8 b8;

#if defined(__cplusplus)
	#define SIZE_ASSERT(type, size) \
		static_assert(sizeof(type) == size, "Expected sizeof " #type " to be " #size " bytes.")
#else
	#define SIZE_ASSERT(type, size) \
		_Static_assert(sizeof(type) == size, "Expected sizeof " #type " to be " #size " bytes.")
#endif

SIZE_ASSERT(u8,  1);
SIZE_ASSERT(u16, 2);
SIZE_ASSERT(u32, 4);
SIZE_ASSERT(u64, 8);

SIZE_ASSERT(s8,  1);
SIZE_ASSERT(s16, 2);
SIZE_ASSERT(s32, 4);
SIZE_ASSERT(s64, 8);

SIZE_ASSERT(s32, 4);
SIZE_ASSERT(s64, 8);

SIZE_ASSERT(b8,  1);

#undef SIZE_ASSERT

#define cast(type) (type)
#define PACKED __attribute__((packed))
