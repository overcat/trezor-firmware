ENTRY(reset_handler)

MEMORY {
  FLASH  (rx)     : ORIGIN = FIRMWARE_START, LENGTH = FIRMWARE_MAXSIZE
  MAIN_STACK (wal): ORIGIN = K_MAIN_STACK_START, LENGTH = K_MAIN_STACK_SIZE
  FB1 (wal)       : ORIGIN = K_FB1_RAM_START, LENGTH = K_FB1_RAM_SIZE
  MAIN_RAM (wal)  : ORIGIN = K_MAIN_RAM_START, LENGTH = K_MAIN_RAM_SIZE
  BOOT_ARGS (wal) : ORIGIN = BOOTARGS_START, LENGTH = BOOTARGS_SIZE
  DMABUF   (wal)  : ORIGIN = DMABUF_RAM_START, LENGTH = DMABUF_RAM_SIZE
}

_stack_section_start = ADDR(.stack);
_stack_section_end = ADDR(.stack) + SIZEOF(.stack);

_data_section_loadaddr = LOADADDR(.data);
_data_section_start = ADDR(.data);
_data_section_end = ADDR(.data) + SIZEOF(.data);

_bss_section_start = ADDR(.bss);
_bss_section_end = ADDR(.bss) + SIZEOF(.bss);

_confidential_section_loadaddr = 0;
_confidential_section_start = 0;
_confidential_section_end = 0;

_bootargs_ram_start = BOOTARGS_START;
_bootargs_ram_end = BOOTARGS_START + BOOTARGS_SIZE;

_codelen = _kernel_flash_end - ORIGIN(FLASH);

SECTIONS {
  .vendorheader : ALIGN(4) {
    KEEP(*(.vendorheader))
  } >FLASH AT>FLASH

  .header : ALIGN(4) {
    . = 1K;
    . = ALIGN(CODE_ALIGNMENT);
  } >FLASH AT>FLASH

  .flash : ALIGN(CODE_ALIGNMENT) {
    KEEP(*(.vector_table));
    . = ALIGN(4);
    *(.text*);
    . = ALIGN(4);
    *(.rodata*);
    . = ALIGN(512);
  } >FLASH AT>FLASH

  .stack : ALIGN(8) {
    . = 8K; /* Exactly 8k allocated for stack. Overflow causes MemManage fault (when using MPU). */
  } >MAIN_STACK

  .data : ALIGN(4) {
    *(.data*);
    . = ALIGN(512);
  } >MAIN_RAM AT>FLASH

  .bss : ALIGN(4) {
    *(.bss*);
    . = ALIGN(4);
  } >MAIN_RAM

  .buf : ALIGN(4) {
    *(.buf*);
    . = ALIGN(4);
  } >DMABUF

  .fb : ALIGN(4) {
    *(.fb1*);
    . = ALIGN(4);
  } >FB1

  .boot_args : ALIGN(8) {
    *(.boot_args*);
    . = ALIGN(8);
  } >BOOT_ARGS

  /DISCARD/ : {
    *(.ARM.exidx*);
  }

  .flash : ALIGN(4) {
    _kernel_flash_end = .;
  } >FLASH
}
