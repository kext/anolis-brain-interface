# Power Supply

As it turns out the power supply might be one of the biggest challenges in the project.

After discarding the micro SD card from the design, the biggest impact on power is the sample rate.
The sample rate simultaneously affects the data rate that has to be transmitted over Bluetooth as well as the power consumption of the AD converter.

The Bluetooth radio needs up to 20mA during transmissions, thus reducing air time is a way to reduce the average current.
Reducing the data rate directly reduces air time.
By including a large enough capacitance on the board it should be possible to smooth out the peak current enough to prevent large voltage drops from the battery.

While zinc-air batteries have very high power densities and low masses, they also have very limited current capabilities.
Smaller zinc-air batteries can supply only about 10mA of current while dropping to a voltage of about 1V.
At this voltage a current of more than 50mA would be needed though.
