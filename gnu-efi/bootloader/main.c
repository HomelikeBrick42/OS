#include <efi.h>
#include <efilib.h>
#include <elf.h>

typedef uint8_t  u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef int8_t  s8;
typedef int16_t s16;
typedef int32_t s32;
typedef int64_t s64;

typedef float  f32;
typedef double f64;

#define cast(type) (type)

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

EFI_FILE *LoadFile(EFI_FILE *directory, CHAR16 *path, EFI_HANDLE imageHandle, EFI_SYSTEM_TABLE *systemTable) {
	EFI_LOADED_IMAGE_PROTOCOL *loadedImage;
	systemTable->BootServices->HandleProtocol(imageHandle, &gEfiLoadedImageProtocolGuid, cast(void**) &loadedImage);

	EFI_SIMPLE_FILE_SYSTEM_PROTOCOL *fileSystem;
	systemTable->BootServices->HandleProtocol(loadedImage->DeviceHandle, &gEfiSimpleFileSystemProtocolGuid, cast(void**) &fileSystem);

	if (directory == NULL) {
		fileSystem->OpenVolume(fileSystem, &directory);
	}

	EFI_FILE *loadedFile;
	EFI_STATUS status = directory->Open(directory, &loadedFile, path, EFI_FILE_MODE_READ, EFI_FILE_READ_ONLY);
	if (status != EFI_SUCCESS) {
		return NULL;
	}

	return loadedFile;
}

int memcmp(const void* a, const void* b, u64 n) {
	for (u64 i = 0; i < n; i++) {
		if ((cast(u8*) a)[i] < (cast(u8*) b)[i]) return -1;
		if ((cast(u8*) a)[i] > (cast(u8*) b)[i]) return 1;
	}

	return 0;
}

Framebuffer gFramebuffer;
Framebuffer *InitializeGOP() {
	EFI_GUID gopGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;

	EFI_GRAPHICS_OUTPUT_PROTOCOL *gop;
	EFI_STATUS status = uefi_call_wrapper(BS->LocateProtocol, 3, &gopGuid, NULL, cast(void**) &gop);
	if (EFI_ERROR(status)) {
		Print(L"Unable to locate GOP\r\n");
		return NULL;
	} else {
		Print(L"GOP located\r\n");
	}

	gFramebuffer.BaseAddress = cast(void*) gop->Mode->FrameBufferBase;
	gFramebuffer.BufferSize = gop->Mode->FrameBufferSize;
	gFramebuffer.Width = gop->Mode->Info->HorizontalResolution;
	gFramebuffer.Height = gop->Mode->Info->VerticalResolution;
	gFramebuffer.PixelsPerScanLine = gop->Mode->Info->PixelsPerScanLine;

	switch (gop->Mode->Info->PixelFormat) {
		case PixelRedGreenBlueReserved8BitPerColor: {
			gFramebuffer.PixelFormat = FramebufferPixelFormat_RGBA;
			Print(L"Pixel Format: RGBA\r\n");
		} break;

		case PixelBlueGreenRedReserved8BitPerColor: {
			gFramebuffer.PixelFormat = FramebufferPixelFormat_BGRA;
			Print(L"Pixel Format: BGRA\r\n");
		} break;

		default: {
			Print(L"Unknown pixel format\r\n");
			return NULL;
		} break;
	}

	return &gFramebuffer;
}

PSF1_Font *LoadPSF1Font(EFI_FILE *directory, CHAR16 *path, EFI_HANDLE imageHandle, EFI_SYSTEM_TABLE *systemTable) {
	EFI_FILE *font = LoadFile(directory, path, imageHandle, systemTable);
	if (font == NULL) {
		return NULL;
	}

	PSF1_Header* fontHeader;
	systemTable->BootServices->AllocatePool(EfiLoaderData, sizeof(PSF1_Header), cast(void**) &fontHeader);
	UINTN size = sizeof(PSF1_Header);
	font->Read(font, &size, fontHeader);

	if (fontHeader->MagicBytes[0] != PSF1_MAGIC_BYTE_0 ||
		fontHeader->MagicBytes[1] != PSF1_MAGIC_BYTE_1) {
		return NULL;
	}

	UINTN glyphBufferSize = fontHeader->CharSize * 256;
	if (fontHeader->Mode == 1) {
		glyphBufferSize = fontHeader->CharSize * 512;
	}

	void* glyphBuffer;
	{
		font->SetPosition(font, sizeof(PSF1_Header));
		systemTable->BootServices->AllocatePool(EfiLoaderData, glyphBufferSize, cast(void**) &glyphBuffer);
		font->Read(font, &glyphBufferSize, glyphBuffer);
	}

	PSF1_Font* finishedFont;
	systemTable->BootServices->AllocatePool(EfiLoaderData, sizeof(PSF1_Font), cast(void**) &finishedFont);
	finishedFont->Header = fontHeader;
	finishedFont->GlyphBuffer = glyphBuffer;

	return finishedFont;
}

typedef struct BootInfo_t {
	Framebuffer *FrameBuffer;
	PSF1_Font *PSF1Font;
	EFI_MEMORY_DESCRIPTOR *MemoryMap;
	UINTN MemoryMapSize;
	UINTN MemoryMapDescriptorSize;
} BootInfo;

EFI_STATUS efi_main(EFI_HANDLE imageHandle, EFI_SYSTEM_TABLE *systemTable) {
	InitializeLib(imageHandle, systemTable);

	EFI_FILE *kernel = LoadFile(NULL, L"kernel.elf", imageHandle, systemTable);
	if (kernel == NULL) {
		Print(L"Could not load kernel\r\n");
		return EFI_NOT_FOUND;
	} else {
		Print(L"Kernel loaded successfuly\r\n");
	}

	Elf64_Ehdr header;
	{
		UINTN fileInfoSize;
		EFI_FILE_INFO *fileInfo;
		kernel->GetInfo(kernel, &gEfiFileInfoGuid, &fileInfoSize, NULL);
		systemTable->BootServices->AllocatePool(EfiLoaderData, fileInfoSize, cast(void**) &fileInfo);
		kernel->GetInfo(kernel, &gEfiFileInfoGuid, &fileInfoSize, (void**) &fileInfo);

		UINTN size = sizeof(Elf64_Ehdr);
		kernel->Read(kernel, &size, &header);
	}

	if (memcmp(&header.e_ident[EI_MAG0], ELFMAG, SELFMAG) != 0 ||
		header.e_ident[EI_CLASS] != ELFCLASS64 ||
		header.e_ident[EI_DATA] != ELFDATA2LSB ||
		header.e_type != ET_EXEC ||
		header.e_machine != EM_X86_64 ||
		header.e_version != EV_CURRENT) {
		Print(L"Kernel format is bad\r\n");
	} else {
		Print(L"Kernel header successfully verified\r\n");
	}

	Elf64_Phdr *programHeaders;
	{
		kernel->SetPosition(kernel, header.e_phoff);
		UINTN size = header.e_phnum * header.e_phentsize;
		systemTable->BootServices->AllocatePool(EfiLoaderData, size, cast(void**) &programHeaders);
		kernel->Read(kernel, &size, programHeaders);
	}

	for (
		Elf64_Phdr *programHeader = programHeaders;
		cast(u8*)programHeader < cast(u8*)programHeaders + header.e_phnum * header.e_phentsize;
		programHeader = cast(Elf64_Phdr*) (cast(u8*) programHeader + header.e_phentsize)
	) {
		switch (programHeader->p_type) {
			case PT_LOAD: {
				u64 pages = (programHeader->p_memsz + 0x1000 - 1) / 0x1000;
				Elf64_Addr segment = programHeader->p_paddr;
				systemTable->BootServices->AllocatePages(AllocateAddress, EfiLoaderData, pages, &segment);

				kernel->SetPosition(kernel, programHeader->p_offset);
				UINTN size = programHeader->p_filesz;
				kernel->Read(kernel, &size, cast(void*) segment);
			} break;

			default: {
			} break;
		}
	}

	Print(L"Kernel loaded\r\n");

	PSF1_Font *newFont = LoadPSF1Font(NULL, L"zap-light16.psf", imageHandle, systemTable);
	if (newFont == NULL) {
		Print(L"Could not load font\r\n");
		return EFI_NOT_FOUND;
	} else {
		Print(L"Font loaded successfuly\r\nChar Size: %d\r\n", newFont->Header->CharSize);
	}

	Framebuffer *newBuffer = InitializeGOP();
	if (newBuffer == NULL) {
		return EFI_UNSUPPORTED;
	}

	Print(L"Base: 0x%llx\r\nSize: 0x%llx\r\nWidth: %llu\r\nHeight: %llu\r\nPixelsPerScanLine: %llu\r\n",
		newBuffer->BaseAddress,
		newBuffer->BufferSize,
		newBuffer->Width,
		newBuffer->Height,
		newBuffer->PixelsPerScanLine
	);

	EFI_MEMORY_DESCRIPTOR *map;
	UINTN mapSize;
	UINTN mapKey;
	UINTN descriptorSize;
	UINT32 descriptorVersion;
	{
		systemTable->BootServices->GetMemoryMap(&mapSize, NULL, &mapKey, &descriptorSize, &descriptorVersion);
		systemTable->BootServices->AllocatePool(EfiLoaderData, mapSize, cast(void**) &map);
		systemTable->BootServices->GetMemoryMap(&mapSize, map, &mapKey, &descriptorSize, &descriptorVersion);
	}

	BootInfo bootInfo = (BootInfo) {
		.FrameBuffer = newBuffer,
		.PSF1Font = newFont,
		.MemoryMap = map,
		.MemoryMapSize = mapSize,
		.MemoryMapDescriptorSize = descriptorSize,
	};

	systemTable->BootServices->ExitBootServices(imageHandle, mapKey);

	(cast(__attribute__((sysv_abi)) void (*)(BootInfo*)) header.e_entry)(&bootInfo);

	return EFI_SUCCESS;
}
