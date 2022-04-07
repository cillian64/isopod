# ISOPOD firmware

This is the firmware for Isopod.

## Firmware functions

* Collect location data from GPS peripheral over UART
* Collect movement/orientation data from IMU over I2C
* Collect battery level from fuel gauge over I2C
* Control addressable LEDs via PWM and GPIO
* Report location to a remote server over HTTPS over Wi-Fi

## LED control algorithm

The LED-control logic is roughly as follows:
* If the battery level is low then do a low-power pattern:
  * Every three seconds flash all LEDs briefly red and otherwise off.
* Otherwise, if we are currently moving, illuminate all the LEDs with a hue
  which depends on current orientation, full saturation, and a value depending
  on gyro rate.
* Otherwise, if we are currently stationary but have moved within the past few
  minutes, do a pattern depending on our current orientation.  Ideas:
  * Smooth rainbow colour fading
  * Random sparkling
  * Radial rainfall with colour spots mixed in
  * TODO: more ideas here
* Otherwise, if we have not moved in a couple of minutes then do the idle
  pattern:
  * Every second, pulse red from the centre out

## Architecture

The firmware is architected roughly as follows:
* GPS, IMU, and battery level data are collected by separate threads
* Their data is written into shared structures which are protected by mutexes
* The main thread implements application logic - it reads from the sensor
  structures and writes to the LED control structure.
* An LED control thread determines the desired LED state, and writes to the
  PWM and GPIO peripherals to control the LEDs.
* A location reporting thread periodically reports the location over HTTPS
  over Wi-Fi.
