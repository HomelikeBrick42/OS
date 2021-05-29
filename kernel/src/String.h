#pragma once

#include "./Typedefs.h"

typedef struct PACKED String_t {
	u8* Data;
	u64 Length;
} String;

#define StringFromLiteral(s) (String) { .Data = cast(u8*) s, .Length = sizeof(s) - 1 }

#if defined(__cplusplus)
extern "C" {
#endif

String StringFromu8(u8 value);
String StringFromu16(u16 value);
String StringFromu32(u32 value);
String StringFromu64(u64 value);

String StringFroms8(s8 value);
String StringFroms16(s16 value);
String StringFroms32(s32 value);
String StringFroms64(s64 value);

String StringFromf32(f32 value, u8 decimals);
String StringFromf64(f64 value, u8 decimals);

#if defined(__cplusplus)
}
#endif
