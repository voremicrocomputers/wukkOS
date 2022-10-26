arch ?= x86_64
kernel := target/$(arch)-custom/debug/libwukkOS.a
iso := build/arch/$(arch)/wukkOS.iso
target ?= $(arch)-custom
final := build/arch/$(arch)/wukkOS.bin
efi_bios := build/arch/$(arch)/OVMF-pure-efi.fd

linker_script := arch/$(arch)/linker.ld
grub_cfg := arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard arch/$(arch)/*.asm)
assembly_object_files := $(patsubst arch/$(arch)/%.asm, \
							build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run iso quick_invalidate

all: $(final) $(iso)

clean:
	@xargo clean
	@rm -rf build

run: $(final) $(iso)
	@qemu-system-$(arch) -bios $(efi_bios) -cdrom $(iso) \
-chardev stdio,id=char0,mux=on,logfile=serial.log,signal=off \
  -serial chardev:char0 -mon chardev=char0

quick_invalidate:
	@echo "quick invalidation"
	@rm -rf build/arch/$(arch)
	@rm -rf $(kernel)

iso: $(iso)

$(iso): $(final) $(grub_cfg)
	@cp OVMF-pure-efi.fd build/arch/$(arch)/OVMF-pure-efi.fd # TODO! remove this, it's only for testing and i don't think we can distribute it
	@mkdir -p isodir/boot/grub
	@cp $(final) isodir/boot/wukkOS.bin
	@cp $(grub_cfg) isodir/boot/grub/grub.cfg
	@grub-mkrescue -o $(iso) isodir
	@rm -rf isodir

$(final): $(kernel) $(linker_script) $(assembly_object_files)
	@ld -n -T $(linker_script) -o $(final) $(assembly_object_files) $(kernel) \
		--gc-sections

$(kernel):
	@RUST_TARGET_PATH=$(shell pwd) xargo build --target $(target) -Zbuild-std=core,alloc

build/arch/$(arch)/%.o: arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@