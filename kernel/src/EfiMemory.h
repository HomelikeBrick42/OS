#pragma once

#include "./Typedefs.h"
#include "./String.h"

typedef struct PACKED EfiMemoryDescriptor_t {
	u32 Type;
	u32 Padding;
	void* PhysicalAddress;
	void* VirtualAddress;
	u64 NumPages;
	u64 Attribs;
} EfiMemoryDescriptor;

extern "C" String EfiMemoryTypeStrings[];
