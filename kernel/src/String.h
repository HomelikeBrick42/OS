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

String StringFromUInt(u64 value);
String StringFromInt(s64 value);
String StringFromFloat(f64 value, u8 decimals);

#if defined(__cplusplus)
}
#endif
