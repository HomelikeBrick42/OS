#include "./Typedefs.h"
#include "./String.h"
#include "./BasicRenderer.hpp"

extern "C" void _start(Framebuffer* framebuffer, PSF1_Font *font) {
	BasicRenderer renderer = BasicRenderer(framebuffer, font);

	renderer.PrintString(StringFromUInt(1234987));
	renderer.PrintChar('\r');
	renderer.PrintChar('\n');
	renderer.PrintString(StringFromInt(-976123));
	renderer.PrintChar('\r');
	renderer.PrintChar('\n');
	renderer.PrintString(StringFromFloat(-123.745, 3));

	while (true) {
		asm ("hlt");
	}
}
