# Storage Data Format

The data file contains all data packets received by the brain interface together with the time the packet was received.
The data file includes all the data packets with a header one after the other.

Each stored packet has the following format:

Field | Byte Offset | Byte Size | Data Type | Description
------|-------------|-----------|-----------|------------
Signature | 0 | 4 | `u32` | The fixed value 0x55daba to identify the file.
Length | 4 | 4 | `u32` | Length of the stored packet in bytes including this header.
Time | 8 | 8 | `u64` | Time this packet was received as number of milliseconds since `1970-01-01T00:00Z`.
Packet Number | 16 | 1 | `u8` | A counter to detect missing packets. Wraps to 0 after 255.
Channel Count | 17 | 1 | `u8` | Number of channels in this packet.
Samples | 18 | Variable | `[u16]` | All samples of the packet as 16 bit integers.

All integer types are in little endian byte order, i.e. least significant byte first.

The samples are stored interleaved with one sample for each channel until there are no more samples.
