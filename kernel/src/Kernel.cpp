#include "./Typedefs.h"
#include "./String.h"
#include "./BasicRenderer.hpp"

extern "C" void _start(Framebuffer* framebuffer, PSF1_Font *font) {
	BasicRenderer renderer = BasicRenderer(framebuffer, font);

	renderer.PrintString(StringFromu64(1234987));
	renderer.PrintChar('\r');
	renderer.PrintChar('\n');
	renderer.PrintString(StringFroms64(-976123));
	renderer.PrintChar('\r');
	renderer.PrintChar('\n');
	renderer.PrintString(StringFromf64(-123.745, 3));

	while (true) {
		asm ("hlt");
	}
}
