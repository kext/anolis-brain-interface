# Throughput

Theoretical throughput at 2M with Data Length Extension and a 50ms connection interval: 1366kbit/s.

If we can keep our data rate below this threshold the SD card is not necessary.
Since it is by far the largest consumer of energy this would make the system a lot smaller and lighter.

## Data Rates

The data rate of the system depends on the sample rate, the channel count and the bits per sample.

Channels | Sample Rate | Bit Depth | Data Rate  | Ok?
---------|-------------|-----------|------------|------
16       | 10kHz       | 16bit     | 2560kbit/s | No
10       | 10kHz       | 16bit     | 1600kbit/s | No
8        | 10kHz       | 16bit     | 1280kbit/s | Maybe
10       | 10kHz       | 10bit     | 1000kbit/s | Yes
10       | 5kHz        | 16bit     | 800kbit/s  | Yes
