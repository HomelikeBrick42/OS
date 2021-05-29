#pragma once

#include "./Typedefs.h"

typedef struct PACKED String_t {
	u8* Data;
	u64 Length;
} String;

#define StringFromLiteral(s) (String) { .Data = cast(u8*) s, .Length = sizeof(s) - 1 }
