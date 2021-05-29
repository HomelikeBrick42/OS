#include "./BasicRenderer.hpp"
#include "./Framebuffer.h"

BasicRenderer::BasicRenderer(Framebuffer *framebuffer, PSF1_Font *font)
	: TargetFramebuffer(framebuffer), PSF1Font(font), CursorX(0), CursorY(0) {
}

void BasicRenderer::PrintChar(u8 chr, u32 color) {
	switch (chr) {
		case '\r': {
			this->CursorX = 0;
		} break;

		case '\n': {
			this->CursorY += this->PSF1Font->Header->CharSize;
		} break;

		default: {
			if ((this->CursorX + (this->PSF1Font->Header->CharSize / 2)) > this->TargetFramebuffer->Width) {
				this->PrintChar('\r', color);
				this->PrintChar('\n', color);
			}

			if ((this->CursorY + this->PSF1Font->Header->CharSize) < this->TargetFramebuffer->Height) {
				u32* pixPtr = cast(u32*) this->TargetFramebuffer->BaseAddress;
				u8* fontPtr = cast(u8*) this->PSF1Font->GlyphBuffer + (chr * this->PSF1Font->Header->CharSize);

				for (u64 y = this->CursorY; y < this->CursorY + this->PSF1Font->Header->CharSize; y++){
					for (u64 x = this->CursorX; x < this->CursorX + (this->PSF1Font->Header->CharSize / 2); x++){
						if ((*fontPtr & (0b10000000 >> (x - this->CursorX))) > 0){
							u32* pixel = cast(u32*) (pixPtr + x + (y * this->TargetFramebuffer->PixelsPerScanLine));
							*pixel = color;
						}
					}
					fontPtr++;
				}
				
				this->CursorX += this->PSF1Font->Header->CharSize / 2;
			}
		} break;
	}
}

void BasicRenderer::PrintString(String string, u32 color) {
	for (u64 i = 0; i < string.Length; i++) {
		this->PrintChar(string.Data[i], color);
	}
}
