#include "./Typedefs.h"
#include "./String.h"
#include "./BasicRenderer.hpp"
#include "./EfiMemory.h"

typedef struct BootInfo_t {
	Framebuffer *FrameBuffer;
	PSF1_Font *PSF1Font;
	EfiMemoryDescriptor *MemoryMap;
	u64 MemoryMapSize;
	u64 MemoryMapDescriptorSize;
} BootInfo;

extern "C" {
	void _start(BootInfo *bootInfo) {
		BasicRenderer renderer = BasicRenderer(bootInfo->FrameBuffer, bootInfo->PSF1Font);

		u64 memoryMapEntries = bootInfo->MemoryMapSize / bootInfo->MemoryMapDescriptorSize;
		for (u64 i = 0; i < memoryMapEntries; i++) {
			EfiMemoryDescriptor* descriptor = cast(EfiMemoryDescriptor*) (cast(u64) bootInfo->MemoryMap + (i * bootInfo->MemoryMapDescriptorSize));
			renderer.PrintString(EfiMemoryTypeStrings[descriptor->Type]);
			renderer.PrintChar(' ');
			renderer.PrintString(StringFromUInt(descriptor->NumPages * 4096 / 1024), 0xFFFF00FF);
			renderer.PrintString(StringFromLiteral("kb"), 0xFFFF00FF);
			renderer.PrintChar('\r');
			renderer.PrintChar('\n');
		}

		while (true) {
			asm ("hlt");
		}
	}
}
