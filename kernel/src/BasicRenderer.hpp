#pragma once

#include "./Typedefs.h"
#include "./String.h"
#include "./Framebuffer.h"
#include "./PSF1_Font.h"

class BasicRenderer {
public:
	BasicRenderer(Framebuffer *framebuffer, PSF1_Font *font);
public:
	void PrintChar(u8 chr, u32 color = 0xFFFFFFFF);
	void PrintString(String string, u32 color = 0xFFFFFFFF);
public:
	u64 CursorX;
	u64 CursorY;
private:
	Framebuffer *TargetFramebuffer;
	PSF1_Font *PSF1Font;
};
