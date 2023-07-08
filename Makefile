# Default to a serial device name that is common in Linux.
DEV_SERIAL ?= /dev/ttyUSB0

TARGET            = aarch64-unknown-none-softfloat
KERNEL_BIN        = kernel8.img
QEMU_BINARY       = qemu-system-aarch64
QEMU_MACHINE_TYPE = raspi3
QEMU_RELEASE_ARGS = -serial stdio -display none
# Assembly mode:
#QEMU_RELEASE_ARGS = -d in_asm -display none
LINKER_FILE       = $(shell pwd)/kernel/src/bsp/raspberrypi/linker.ld
RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53
BIN_NAME          = iris-os

# Export for build.rs
export LINKER_FILE

RUSTFLAGS          = -C link-arg=-T$(LINKER_FILE) $(RUSTC_MISC_ARGS)
RUSTFLAGS_PEDANTIC = $(RUSTFLAGS) #-D warnings #-D missing_docs

COMPILER_ARGS = --target=$(TARGET) \
    --release

DOCKER_IMAGE         = rustembedded/osdev-utils
DOCKER_CMD           = docker run -it --rm -v $(shell pwd):/work/tutorial -w /work/tutorial
DOCKER_ARG_DIR_UTILS = -v $(shell pwd)/../utils:/work/utils
DOCKER_ARG_DEV       = --privileged -v /dev:/dev
DOCKER_QEMU = $(DOCKER_CMD) $(DOCKER_IMAGE)
DOCKER_CMD_DEV = $(DOCKER_CMD) $(DOCKER_ARG_DEV)
DOCKER_CHAINBOOT = $(DOCKER_CMD_DEV) $(DOCKER_ARG_DIR_UTILS) $(DOCKER_IMAGE)


RUSTC_CMD   = cargo rustc --manifest-path kernel/Cargo.toml $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS)
CLIPPY_CMD  = cargo clippy --manifest-path kernel/Cargo.toml $(COMPILER_ARGS)
CHECK_CMD   = cargo check $(COMPILER_ARGS)
TEST_CMD    = cargo test
OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary

KERNEL_ELF = target/$(TARGET)/release/$(BIN_NAME)

EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)
EXEC_MINIPUSH = ruby utils/minipush.rb

.PHONY: all $(KERNEL_ELF) $(KERNEL_BIN) doc qemu clippy clean readelf objdump nm check

all: $(KERNEL_BIN)

build: $(KERNEL_BIN)

$(KERNEL_ELF):
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(RUSTC_CMD)

$(KERNEL_BIN): $(KERNEL_ELF)
	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)

chainboot: $(KERNEL_BIN)
	@$(DOCKER_CHAINBOOT) $(EXEC_MINIPUSH) $(DEV_SERIAL) $(KERNEL_BIN)

doc:
	$(DOC_CMD) --document-private-items --open

qemu: $(KERNEL_BIN)
	@$(DOCKER_QEMU) $(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)

clippy:
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

test:
	RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(TEST_CMD)
clean:
	rm -rf target $(KERNEL_BIN)

readelf: $(KERNEL_ELF)
	readelf -a $(KERNEL_ELF)

objdump: $(KERNEL_ELF)
	rust-objdump --arch-name aarch64 --disassemble --demangle --no-show-raw-insn \
	    --print-imm-hex $(KERNEL_ELF)

nm: $(KERNEL_ELF)
	rust-nm --demangle --cprint-size $(KERNEL_ELF) | sort

# For rust-analyzer
check:
	@RUSTFLAGS="$(RUSTFLAGS)" $(CHECK_CMD) --message-format=json
