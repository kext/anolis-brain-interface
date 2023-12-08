MEMORY {
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the nRF52840 with Softdevice S140 7.3.0 */
  FLASH : ORIGIN = 0x00000000 + 0x27000, LENGTH = 1024K - 0x27000
  RAM : ORIGIN = 0x20000000 + 0x35c8, LENGTH = 256K - 0x35c8
}
