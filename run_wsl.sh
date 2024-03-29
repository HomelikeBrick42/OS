cmd.exe /c cargo build --release

cp /usr/share/OVMF/OVMF_CODE.fd .
cp /usr/share/OVMF/OVMF_VARS.fd .

mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/release/os.efi esp/efi/boot/bootx64.efi

cmd.exe /c qemu-system-x86_64 -m 256M \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp
