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

typedef struct Framebuffer_t {
	void* BaseAddress;
	u64 BufferSize;
	u64 Width;
	u64 Height;
	u64 PixelsPerScanLine;
} Framebuffer;

void _start(Framebuffer* framebuffer) {
	u64 y = 50;
	u64 bytesPerPixel = 4;

	for (u64 x = 0; x < framebuffer->Width / 2 * bytesPerPixel; x += bytesPerPixel) {
		u32* pixel = cast(u32*) (x + (y * framebuffer->PixelsPerScanLine * bytesPerPixel) + framebuffer->BaseAddress);
		*pixel = 0xFFFFFF00; // ARGB
	}
}
