arch ?= x86_64
kernel := target/$(arch)-custom/debug/wukkOS
iso := build/arch/$(arch)/wukkOS.iso
target ?= $(arch)-custom
final := build/arch/$(arch)/wukkOS.bin
initwukko := byob/initwukko.far
initwukko_from := initwukko
efi_bios := build/arch/$(arch)/OVMF-pure-efi.fd
kernel_flags :=
RUST_FLAGS :=
gcc ?= gcc
ld ?= ld
far-rs ?= $(shell which far-rs)

linker_script := arch/$(arch)/linker.ld
bootloader_cfg := arch/$(arch)/limine.cfg
assembly_source_files := $(wildcard arch/$(arch)/*.asm)
assembly_object_files := $(patsubst arch/$(arch)/%.asm, \
							build/arch/$(arch)/%.o, $(assembly_source_files))


ifeq "$(arch)" "x86_64"
kernel_flags += --features "f_limine"
endif
ifeq "$(arch)" "ppc32"
endif

.PHONY: all clean run iso quick_invalidate build_no_iso

all: $(final) $(iso)

build_no_iso: $(final)

clean:
	@xargo clean
	@rm -rf build

run: $(final) $(iso)
	@qemu-system-$(arch) -bios $(efi_bios) -cdrom $(iso) -d int -D qemulog.log \
-chardev stdio,id=char0,mux=on,logfile=serial.log,signal=off \
  -serial chardev:char0 -mon chardev=char0 -m 512M

quick_invalidate:
	@echo "quick invalidation"
	@rm -rf build/arch/$(arch)
	@rm -rf $(kernel)

iso: $(iso)

$(iso): $(final) $(grub_cfg)
	@cp OVMF-pure-efi.fd build/arch/$(arch)/OVMF-pure-efi.fd # TODO! remove this, it's only for testing and i don't think we can distribute it
	@mkdir -p isodir/boot
	@cp $(final) isodir/boot/wukkOS.bin
	@cp $(bootloader_cfg) isodir/boot/limine.cfg
	@cp byob/limine.sys byob/limine-cd.bin byob/limine-cd-efi.bin $(initwukko) isodir/boot/
	@xorriso -as mkisofs -b boot/limine-cd.bin \
	-no-emul-boot -boot-load-size 4 -boot-info-table \
	--efi-boot boot/limine-cd-efi.bin \
	-efi-boot-part --efi-boot-image --protective-msdos-label \
	isodir -o $(iso)
	@rm -rf isodir
	@byob/limine-deploy $(iso)

$(final): $(kernel) $(linker_script) $(initwukko)
	@mkdir -p $(shell dirname $@)
	@cp $(kernel) $(final)
	#@$(ld) -n -T $(linker_script) -o $(final) $(kernel) \
	#	--gc-sections

$(kernel):
	@RUSTFLAGS="$(RUST_FLAGS)" RUST_TARGET_PATH="$(shell pwd)" cargo +nightly build --target $(target) -Zbuild-std=core,alloc $(kernel_flags)

$(initwukko):
	@mkdir -p $(shell dirname $@)
	@$(far-rs) create $(initwukko) initwukko/*

build/arch/$(arch)/%.o: arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@