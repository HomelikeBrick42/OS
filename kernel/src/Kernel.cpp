#include "./Typedefs.h"
#include "./String.h"
#include "./BasicRenderer.hpp"

extern "C" void _start(Framebuffer* framebuffer, PSF1_Font *font) {
	String string = StringFromLiteral("Hello\nKernel");

	BasicRenderer renderer(framebuffer, font);
	renderer.CursorX = 50;
	renderer.CursorY = 120;
	renderer.PrintString(string);

	while(1) {
		asm ("hlt");
	}
}
