#include "./EfiMemory.h"

extern "C" {
	String EfiMemoryTypeStrings[] = {
		StringFromLiteral("EfiReservedMemoryType"),
		StringFromLiteral("EfiLoaderCode"),
		StringFromLiteral("EfiLoaderData"),
		StringFromLiteral("EfiBootServicesCode"),
		StringFromLiteral("EfiBootServicesData"),
		StringFromLiteral("EfiRuntimeServicesCode"),
		StringFromLiteral("EfiRuntimeServicesData"),
		StringFromLiteral("EfiConventionalMemory"),
		StringFromLiteral("EfiUnusableMemory"),
		StringFromLiteral("EfiACPIReclaimedMemory"),
		StringFromLiteral("EfiACPIMemoryNVS"),
		StringFromLiteral("EfiMemoryMappedIO"),
		StringFromLiteral("EfiMemoryMappedIOPortSpace"),
		StringFromLiteral("EfiPalCode"),
	};
}
