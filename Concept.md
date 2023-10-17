# Concept

This project aims to build a device that is able to measure the bioelectric potential in the brains of Anolis lizards.
The device should be attached to the lizard and impact it as little as possible.

The data should be sampled at a decently high rate on multiple channels and then both be saved for later analysis and also displayed in realtime.

The realtime display is achieved by sending the data via Bluetooth to a connected PC.
Depending on the sample rate and the number of channels this is either all of the data or a reduced form.

In case only a reduced form can be sent in realtime, the data storage has to happen on the device itself.
A micro SD card would be the obvious choice for that.
Otherwise the data can just be recorded on the PC.

## Components

### Analogue Frontend

The analogue frontend is implemented with the Intan RHD2216 ADC.
It has 16 analogue differential inputs and an SPI connection to retrieve the data.

Depending on the configuration the RHD2216 draws between about 1mA to 15mA from a 3.3V power supply.

### Bluetooth Interface

The Bluetooth connectivity will be realized with the nRF52840 SoC by Nordic Semiconductor.
The SoC is powered by a 64MHz Arm Cortex M4F processor, which will not only handle the Bluetooth connection but will also run all the application code.

The highest contributor to power consumption is the Bluetooth radio.
Depending on the usage pattern the whole module should draw no more than about 20mA from the 3.3V supply while continuously transmitting data.

One of the smallest modules containing the nRF52840 is the Setebos-I by Würth Electronic.
It already contains the antenna circuit and the CPU.

For programming, a development kit with an SWD debugger is needed to flash the program onto the SoC.
The obvious choice here is the nRF52840-DK by Nordic Semiconductor.

To connect the module to a PC a Bluetooth dongle is needed.
A good choice will most likely be the nRF52840-Dongle by Nordic Semiconductor.
It uses the same nRF52840 SoC and thus supports the same level of Bluetooth communication including the new 2M PHY for higher data rate introduced in Bluetooth version 5.

### Data Storage

Data storage will happen on a micro SD card if the data rate is too high for realtime transmission.
In addition to the faster SD mode it can also be used via SPI which allows it to be connected directly to the processor.

If a micro SD card is required it will be the biggest consumer of power with peaks in excess of 100mA.
The power supply must therefore be able to handle this current and should be able to supply at least 300mA to leave enough capacity for the other components.

### Power Supply

The system must be powered by batteries to enable a wireless operation.
Popular choices here are either rechargeable LiPo batteries often found in small model toys or Zinc-Air primary cells which have among the highest energy densities by both mass and weight, often found in hearing aids.

When using LiPo batteries, a charging circuit and voltage converter is required to keep the voltage at a steady 3.3V and to charge the battery.

For use with Zinc-Air coin cells a boost converter is needed to step up the 1.45V up to the required 3.3V.
A good choice might be the TPS610994 by Texas Instruments.
It has a very low quiescent current and can use the battery until it has dropped to 0.7V.
An even better choice might be the TPS61299 but it is not widely available yet.

## Requirements

A number of requirements have to be specified to properly dimension the project.

### Channel Count

The channel count not only influences the data rate but also dictates how much board space is needed for the connectors and the electrodes.
The maximum channel count possible with the RHD2216 is 16 channels.

Also important is if the channels should share a common reference or if each channel is using the full differential input.

### Sample Rate

The sample rate has the biggest influence on the overall data rate.
If the data rate is too high the micro SD card for data storage can not be omitted from the system.
If it is lower the complete data can be transferred via Bluetooth to be recorded directly on the PC.

### Bit Depth

The RHD2216 by default has a bit depth of 16 bits per sample.
This has a lower overall impact, but reducing the bit depth by either limiting the input range by removing high order bits or by reducing the resolution by removing low order bits can significantly decrease the data rate.
Another possibility would be using dynamic range compression, i.e. companding the signal by applying a non-linear function before reduction.

### Operating Time

The required operating time dictates how many batteries are required and how big those have to be.
A longer operating time means bigger or more batteries and thus heavier weight and larger dimensions of the overall system.

## Part List

The following parts are needed for the development.

Part               | Supplier | Order Number
-------------------|----------|---------------------
nRF52840-DK        | Mouser   | 949-NRF52840-DK
nRF52840-Dongle    | Mouser   | 949-NRF52840-DONGLE
Setebos-I          | Farnell  | 3868806
TPS610994YFFR      | Mouser   | 595-TPS610994YFFR
VLS201610CX-2R2M-1 | Mouser   | 810-VLS201610CX2R2M1
