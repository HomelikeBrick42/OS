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

EFI_STATUS efi_main(EFI_HANDLE imageHandle, EFI_SYSTEM_TABLE *systemTable) {
	InitializeLib(imageHandle, systemTable);

	EFI_FILE *kernel = LoadFile(NULL, L"kernel.elf", imageHandle, systemTable);
	if (kernel == NULL) {
		Print(L"Could not load kernel\r\n");
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

	if (
		memcmp(&header.e_ident[EI_MAG0], ELFMAG, SELFMAG) != 0 ||
		header.e_ident[EI_CLASS] != ELFCLASS64 ||
		header.e_ident[EI_DATA] != ELFDATA2LSB ||
		header.e_type != ET_EXEC ||
		header.e_machine != EM_X86_64 ||
		header.e_version != EV_CURRENT
	) {
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

	int kernelResult = (cast(__attribute__((sysv_abi)) int (*)(void)) header.e_entry)();
	Print(L"%d\r\n", kernelResult);

	return EFI_SUCCESS;
}
