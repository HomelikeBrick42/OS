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

typedef struct __attribute__((packed)) String_t {
	u8* Data;
	u64 Length;
} String;

#define StringFromLiteral(s) (String) { .Data = cast(u8*) s, .Length = sizeof(s) - 1 }

typedef enum FramebufferPixelFormat_t {
	FramebufferPixelFormat_RGBA,
	FramebufferPixelFormat_BGRA,
} FramebufferPixelFormat;

typedef struct __attribute__((packed)) Framebuffer_t {
	void* BaseAddress;
	u64 BufferSize;
	u64 Width;
	u64 Height;
	u64 PixelsPerScanLine;
	FramebufferPixelFormat PixelFormat;
} Framebuffer;

#define PSF1_MAGIC_BYTE_0 0x36
#define PSF1_MAGIC_BYTE_1 0x04

typedef struct __attribute__((packed)) PSF1_Header_t {
	u8 MagicBytes[2];
	u8 Mode;
	u8 CharSize;
} PSF1_Header;

typedef struct __attribute__((packed)) PSF1_Font_t {
	PSF1_Header *Header;
	void* GlyphBuffer;
} PSF1_Font;

void WriteChar(Framebuffer *framebuffer, PSF1_Font *font, u32 color, u8 chr, u64 xOffset, u64 yOffset);
void WriteString(Framebuffer *framebuffer, PSF1_Font *font, u32 color, String string, u64 xOffset, u64 yOffset);

void _start(Framebuffer* framebuffer, PSF1_Font *font) {
	WriteString(framebuffer, font, 0xFFFFFFFF, StringFromLiteral("Hello\r\nKernel"), 0, 0);
}

void WriteChar(Framebuffer *framebuffer, PSF1_Font *font, u32 color, u8 chr, u64 xOffset, u64 yOffset) {
	// TODO: TranslateColor

	u32* pixPtr = cast(u32*) framebuffer->BaseAddress;
    u8* fontPtr = font->GlyphBuffer + (chr * font->Header->CharSize);
    for (u64 y = yOffset; y < yOffset + font->Header->CharSize; y++){
        for (u64 x = xOffset; x < xOffset + (font->Header->CharSize / 2); x++){
            if ((*fontPtr & (0b10000000 >> (x - xOffset))) > 0){
				u32* pixel = cast(u32*) (pixPtr + x + (y * framebuffer->PixelsPerScanLine));
				*pixel = color;
			}
        }
        fontPtr++;
    }
}

void WriteString(Framebuffer *framebuffer, PSF1_Font *font, u32 color, String string, u64 xOffset, u64 yOffset) {
	for (u64 i = 0; i < string.Length; i++) {
		switch (string.Data[i]) {
			case '\r': {
				xOffset = 0;
			} break;

			case '\n': {
				yOffset += font->Header->CharSize;
			} break;

			default: {
				WriteChar(framebuffer, font, color, string.Data[i], xOffset, yOffset);
				xOffset += font->Header->CharSize / 2;
				if (xOffset + font->Header->CharSize / 2 > framebuffer->Width) {
					xOffset = 0;
					yOffset += font->Header->CharSize;
				}
			} break;
		}
	}
}
