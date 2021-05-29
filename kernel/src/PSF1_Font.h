#pragma once

#include "./Typedefs.h"

#define PSF1_MAGIC_BYTE_0 0x36
#define PSF1_MAGIC_BYTE_1 0x04

typedef struct PACKED PSF1_Header_t {
	u8 MagicBytes[2];
	u8 Mode;
	u8 CharSize;
} PSF1_Header;

typedef struct PACKED PSF1_Font_t {
	PSF1_Header *Header;
	void* GlyphBuffer;
} PSF1_Font;
