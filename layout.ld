ENTRY(_reset);

MEMORY
{
  FLASH (rx)      : ORIGIN = 0x08000000, LENGTH = 128K
  RAM (xrw)       : ORIGIN = 0x20000000, LENGTH = 8K
  MEMORY_B1 (rx)  : ORIGIN = 0x60000000, LENGTH = 0K
}

SECTIONS
{
  .text :
  {
    /* Vector table */
    LONG(ORIGIN(RAM) + LENGTH(RAM))
    LONG(_reset + 1);
    KEEP(*(.rodata._EXCEPTIONS));

    /* Reset handler */
    _reset = .;
    *(.text._reset)

    *(.text*)
  } > FLASH

  /DISCARD/ :
  {
    *(.ARM.exidx*)
    *(.note.gnu.build-id*)
  }
}
