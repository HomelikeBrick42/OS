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

#define cast(type) (type)

typedef enum FramebufferPixelFormat_t {
	FramebufferPixelFormat_RGBA,
	FramebufferPixelFormat_BGRA,
} FramebufferPixelFormat;

typedef struct Framebuffer_t {
	void* BaseAddress;
	u64 BufferSize;
	u64 Width;
	u64 Height;
	u64 PixelsPerScanLine;
	FramebufferPixelFormat PixelFormat;
} Framebuffer;

void WritePixel(Framebuffer *framebuffer, u64 x, u64 y, u8 r, u8 g, u8 b, u8 a) {
	switch (framebuffer->PixelFormat) {
		case FramebufferPixelFormat_RGBA: {
			u8* pixel = cast(u8*) ((x * 4) + (y * framebuffer->PixelsPerScanLine * 4) + framebuffer->BaseAddress);
			*pixel++ = r;
			*pixel++ = g;
			*pixel++ = b;
			*pixel++ = a;
		} break;

		case FramebufferPixelFormat_BGRA: {
			u8* pixel = cast(u8*) ((x * 4) + (y * framebuffer->PixelsPerScanLine * 4) + framebuffer->BaseAddress);
			*pixel++ = b;
			*pixel++ = g;
			*pixel++ = r;
			*pixel++ = a;
		} break;

		default: {
		} break;
	}
}

void _start(Framebuffer* framebuffer) {
	u64 y = 50;

	for (u64 x = 0; x < framebuffer->Width / 2; x++) {
		WritePixel(framebuffer, x, y, 0xFF, 0xFF, 0x00, 0xFF);
	}
}
