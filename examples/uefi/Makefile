MODE ?= debug
TARGET := target
QEMU_FD_DIR := $(shell dirname $(shell which qemu-system-x86_64))/../share/qemu
CODE_FD := ${QEMU_FD_DIR}/edk2-x86_64-code.fd
VARS_FD := ${QEMU_FD_DIR}/edk2-i386-vars.fd
EFI := $(TARGET)/x86_64-unknown-uefi/$(MODE)/uefi.efi
ESP := $(TARGET)/x86_64-unknown-uefi/$(MODE)/esp
BUILD_ARGS := -Z build-std=core,alloc
QEMU_ARGS := -net none -nographic -smp 4
OBJDUMP := rust-objdump

ifeq (${MODE}, release)
	BUILD_ARGS += --release
endif

ifeq ($(ACCEL), 1)
ifeq ($(shell uname), Darwin)
QEMU_ARGS += -accel hvf
else
QEMU_ARGS += -accel kvm
endif
endif

build:
	cargo build $(BUILD_ARGS)

run: build
	mkdir -p $(ESP)/EFI/Boot
	cp $(EFI) $(ESP)/EFI/Boot/BootX64.efi
	qemu-system-x86_64 \
		-drive if=pflash,format=raw,file=${CODE_FD},readonly=on \
		-drive if=pflash,format=raw,file=${VARS_FD} \
		-drive format=raw,file=fat:rw:${ESP} \
		$(QEMU_ARGS)

header:
	$(OBJDUMP) -h $(EFI) | less

asm:
	$(OBJDUMP) -d $(EFI) | less
