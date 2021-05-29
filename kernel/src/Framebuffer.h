#pragma once

#include "./Typedefs.h"

typedef enum FramebufferPixelFormat_t {
	FramebufferPixelFormat_RGBA,
	FramebufferPixelFormat_BGRA,
} FramebufferPixelFormat;

typedef struct PACKED Framebuffer_t {
	void* BaseAddress;
	u64 BufferSize;
	u64 Width;
	u64 Height;
	u64 PixelsPerScanLine;
	FramebufferPixelFormat PixelFormat;
} Framebuffer;
