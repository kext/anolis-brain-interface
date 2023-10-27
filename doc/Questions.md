# Questions

## For the Biologists

### How many electrodes are needed?

This influences how many channels are needed on the PCB.

**Answer:** As many as possible.
Let's start with 8 channels, i.e. 9 electrodes.

### What type of electrodes will be used?

How big are they?
How are they attached?
How long and thick are the wires?

**Answer:** Wire diameter 75µm.

[FE6215 Stainless Steel AISI 316L Insulated Wire](https://www.advent-rm.com/en-GB/Search?SearchTerm=FE6215)

An alternative would be

### What are the frequencies of the signals?

This determines the sample rate and cut off frequency used for the ADC.

**Answer:** Maybe 2kHz is enough.
We will go as high as possible with 8 channels.
5kHz should be doable.

### How and where should the board be attached?

The board could be attached either on the head or on the back of the lizard.
Will it be glued or strapped to the lizard?

We could include slits at the sides of the PCB to attach a strapping belt.

**Answer:** Probably glueing.

*Note:* Include slits between the components to allow for more flexing of the board.

### How long will the experiments last?

This determines how much battery capacity is needed.

**Answer:** Saving weight is paramount.
We go with the lightest battery that can still power the system.

### Which species of lizard will be examined?

How big are they?
What is their weight?
How much can they carry without being impacted?

**Answer:** Anolis Carolinensis.
About 6-7g.
They can carry maybe 3-4g.

## For us

### Which battery type to use?

We could use either lithium button cells, zinc-air button cells or LiPo rechargeable batteries.

**Answer:** We should use the lightest battery that can supply the system.

### Can a zinc-air battery supply the needed current?

When using a micro SD card, the power supply needs to be able to supply peak currents of 200mA.
With the lower voltage of the zinc-air button cell of 1.45V vs the 3.3V operating voltage this current increases to about 450mA.

We should conduct an experiment where we discharge a number of coin cells with different currents both pulsed and continuously.
This also can show us the expected operation time for each type of battery.
