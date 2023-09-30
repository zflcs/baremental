MODE := "release"
TARGET := "target/riscv64gc-unknown-none-elf/"  + MODE + "/"

ELF := TARGET + "baremental"
BIN := TARGET + "baremental.bin"

# Binutils
OBJDUMP := "rust-objdump --arch-name=riscv64"
OBJCOPY := "rust-objcopy --binary-architecture=riscv64"

kernel:
	LOG=DEBUG cargo build --release

build: kernel
	{{OBJCOPY}} {{ELF}} --strip-all -O binary {{BIN}}


upload: build
	cd ./opensbi && make CROSS_COMPILE=riscv64-unknown-elf- PLATFORM=axu15eg && \
	scp build/platform/axu15eg/firmware/fw_payload.bin axu15eg:~
	ssh axu15eg "./start_rocket.sh"

clean:
	cargo clean
