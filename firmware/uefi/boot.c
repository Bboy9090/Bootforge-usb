/* Standalone, no-disk-write x86_64 UEFI certification marker. */
typedef unsigned long long UINTN;
typedef unsigned long long EFI_STATUS;
typedef void *EFI_HANDLE;
typedef unsigned short CHAR16;
#define EFIAPI __attribute__((ms_abi))
typedef EFI_STATUS (EFIAPI *EFI_TEXT_STRING)(void *, const CHAR16 *);
typedef struct {
  void *Reset; void *OutputStringPlaceholder;
} UNUSED;
typedef struct {
  void *Reset;
  EFI_TEXT_STRING OutputString;
} EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL;
typedef struct {
  char reserved[64];
  EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *ConOut;
} EFI_SYSTEM_TABLE;
EFI_STATUS EFIAPI efi_main(EFI_HANDLE image, EFI_SYSTEM_TABLE *system) {
  (void)image;
  static const CHAR16 marker[] = { 'I','N','F','I','N','I','T','E','B','O','O','T',' ','U','E','F','I',' ','C','E','R','T',' ','O','K','\r','\n',0 };
  if (!system || !system->ConOut || !system->ConOut->OutputString) return 2;
  return system->ConOut->OutputString(system->ConOut, marker);
}
