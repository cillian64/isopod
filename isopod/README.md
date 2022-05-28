# ISOPOD firmware

This is the firmware for Isopod.

## Simulator / visualiser
Normally the software is built to run on the raspberry pi hardware using the
`build_and_remote_run.sh` script.  However for pattern development it's
possible to run the software on a PC (with no sensor or GPS or LED access)
and view the LED status using the web visualiser.  To run the software on
a PC, run, on linux:

```cargo run --target=x86_64-unknown-linux-gnu --no-default-features --bin isopod```

Or on MacOS:

```cargo run --target=x86_64-apple-darwin --no-default-features --bin isopod```

Then open the [sim/sim.html](sim/sim.html) file in your browser.  The
visualiser can be connected to either the actual raspberry pi or to the local
simulator by toggling the "local sim" / "hardware" buttons in the top left
corner.

## Architecture
### Firmware functions
* Collect location data from GPS peripheral over UART
* Collect movement/orientation data from IMU over I2C
* Collect battery level from fuel gauge over I2C
* Control addressable LEDs via PWM and GPIO
* Report location to a remote server over HTTPS over Wi-Fi

### LED control algorithm
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

### Architecture
The firmware is architected roughly as follows:
* GPS, IMU, and battery level data are collected by separate threads
* Their data is written into shared structures which are protected by mutexes
* The main thread implements application logic - it reads from the sensor
  structures and writes to the LED control structure.
* An LED control thread determines the desired LED state, and writes to the
  PWM and GPIO peripherals to control the LEDs.
* A location reporting thread periodically reports the location over HTTPS
  over Wi-Fi.

## Setup

Install rust with default setup:

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Install some more things, some only required for cross-compilation

sudo apt install llvm build-essential gcc-arm-linux-gnueabihf libclang-dev gcc-multilib
