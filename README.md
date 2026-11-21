# Crux

**An embedded navigation subsystem in Rust, built as a sandbox for `nav-core`.**

Crux reads inertial, magnetic, barometric, and (soon) GPS sensors on a small microcontroller and works toward fusing them into a single navigation state: attitude, heading, altitude, vertical speed, and position. It is also the sandbox where the hardware-independent math behind that, called **`nav-core`**, is developed and tested before being reused in future projects.

> **Status: prototype.** Crux runs on a Raspberry Pi Pico 2 W with hobby-grade sensors. It is a learning and experimentation platform. It is **not certified or validated, and must not be used for real navigation or any safety-critical purpose.**

Progress is tracked on the [project board](https://github.com/users/alexwood4real/projects/8) and in the [issues](https://github.com/alexwood4real/Crux/issues).

---

## Background

This project started as a barometer: a Raspberry Pi Pico 2 W with a BME280 sensor, written in Rust. I wanted to learn Rust by building something related to my career. There are plenty of Python and C/C++ examples for the Pico 2 W, but Rust support for the RP2350 is still young, so there are few tutorials for what this project does. The barometer worked, and it grew into Crux: a navigation sandbox that adds an IMU, a magnetometer, and (soon) GPS and a display.

## Goal

Build a small, understandable, well-tested navigation stack that:

1. runs on cheap microcontrollers today and can move to more capable hardware later,
2. keeps the navigation math completely separate from the hardware, so the same code can be reused on any board, and
3. grows in steps through working examples: a barometer first (done), then an attitude and heading demo, and eventually vehicles such as a drone.

## Crux and `nav-core`

Crux has two jobs:

- **The firmware** that reads real sensors on real hardware and shows what works and what breaks.
- **A sandbox for `nav-core`.** Anything that does not need to know about a specific chip, pin, or board (sensor sample types, calibration, weather and air-data math, attitude and heading filters, fusion) is developed as `nav-core`, so it can be lifted out and used by future projects on other hardware. Crux is where that code gets its first real-world testing.

## What is an embedded navigation subsystem?

A navigation system answers a few questions continuously: *Which way am I pointing? How fast and in which direction am I moving? How high am I? Where am I?*

"Embedded" means the answer is produced by a small, self-contained unit (sensors, a processor, and software) built into a vehicle or instrument, and other systems consume its output. Commercial examples include embedded GPS/INS units for aircraft, which report values such as position, velocity, attitude, heading, altitude, and body rates, and navigation built into road vehicles (see [References](#references)).

Crux follows the same idea at hobbyist scale, using sensors that each cover a weakness of the others:

| Sensor | Measures | Strength | Weakness |
|---|---|---|---|
| **IMU** (MPU6050: accelerometer + gyro) | Acceleration and rotation rate | Fast, smooth short-term motion | Gyro drifts over time; accelerometer is noisy under vibration |
| **Magnetometer** (QMC5883P) | Magnetic field direction | Drift-free heading reference | Disturbed by nearby metal, wires, and currents |
| **Barometer** (BME280) | Pressure, temperature, humidity | Altitude, vertical speed, air properties | Slow; weather shifts the reading |
| **GPS** (PA1616S, planned) | Position, ground speed, time | Absolute position, no drift | Slow update rate, needs sky view |

**Sensor fusion** combines these so each corrects the others. For example, the gyro gives smooth attitude changes while the accelerometer slowly pulls the estimate back toward true "down", and the magnetometer anchors heading.

## Why Rust?

Embedded navigation code runs unattended, talks to hardware directly, and has to keep working. Rust fits that job well:

- **Memory safety without a garbage collector.** Buffer overruns and use-after-free bugs, the classic embedded failure modes, are compile errors rather than field failures.
- **Safe concurrency.** The compiler rejects data races between tasks and interrupts, which matters when several sensors are read at different rates.
- **Hardware ownership in the type system.** In Embassy, each pin and peripheral is an owned value, so two drivers cannot accidentally claim the same pin or bus.
- **Zero-cost abstractions and `no_std`.** The same code can run on a small chip with no operating system and no heap.
- **A portable driver ecosystem.** The `embedded-hal` traits mean a sensor driver written once works across chip families, which makes moving to other boards realistic.
- **Async without an RTOS.** [Embassy](https://embassy.dev) provides cooperative async tasks on bare metal, which suits "read several sensors, process, log" well.
- **Testable pure logic.** Math that has no hardware dependencies can be tested on a laptop against recorded sensor data.

Honest trade-offs: Rust has a learning curve, some embedded driver crates are young and need to be checked against real hardware, and certification tooling for safety-critical work is still maturing compared with C and Ada.

## Where it could be used

Crux is a prototype today, but the design is aimed at applications like these:

- **Unmanned vehicles**: attitude, heading, and altitude estimates for drones and other small aircraft.
- **Atmospheric sensing**: balloon, glider, and weather-station payloads that combine altitude with temperature, pressure, and humidity.
- **Ground vehicles and robots**: heading and tilt for rovers, AGVs, and motorcycles.
- **Marine and handheld instruments**: compasses, inclinometers, and heading displays.
- **Instrument displays**: an attitude and heading display driven by the fused navigation state is a planned example.
- **Education and research**: a readable reference for comparing attitude filters.

## Current status

| Area | Status |
|---|---|
| Barometer (BME280 over SPI) with psychrometric and altitude calculations | Working |
| Wi-Fi (CYW43) connection and status LED | Working |
| IMU (MPU6050 over I2C): accelerometer and gyro | Reading values; calibration and processing in progress |
| Magnetometer (QMC5883P over I2C) | Reading values; calibration and processing in progress |
| Sensor fusion (attitude, heading) | Up next |
| GPS (PA1616S over UART) | Planned |
| Display (ST7789 over SPI) and an on-screen attitude demo | Planned |
| Vehicle use cases (for example a drone), additional hardware targets | Future |

## Hardware

The prototype runs on a **Raspberry Pi Pico 2 W** (RP2350 with Wi-Fi). There is a real difference between a Pico 2 W and a plain Pico 2: the Pico 2 W has the CYW43439 Wi-Fi chip, and the onboard LED is wired to that chip, not directly to the main processor. If you use this repo, make sure you have the **W** version.

The first parts were a [Pico 2 W](https://a.co/d/0j32BlzX) and a [BME280 sensor](https://a.co/d/0iv6Qy2b), both ordered from Amazon. The BME280 breakout needs header pins soldered on.

Each sensor has its own bus, so there are no shared-bus conflicts:

| Device | Bus | GPIO |
|---|---|---|
| BME280 | SPI0 | SCK GP18, MOSI GP19, MISO GP16, CS GP17 |
| MPU6050 | I2C0 | SDA GP4, SCL GP5 |
| QMC5883P | I2C1 | SDA GP14, SCL GP15 |
| PA1616S GPS (planned) | UART0 | TX GP0, RX GP1 |
| ST7789 display (planned) | SPI1 | SCK GP10, MOSI GP11, CS GP9, DC GP8, RST GP12, BL GP13 |

All parts run at 3.3 V. GP23, GP24, GP25, and GP29 are used internally by the Wi-Fi chip and must not be used. The interactive pinout at [pico2w.pinout.xyz](https://pico2w.pinout.xyz) is a handy reference.

## Software

Development so far has been on a 2021 MacBook Pro (M1 Max) in VS Code. If you have not installed Rust, start at the official [Rust](https://rust-lang.org) site, and the [Rustlings](https://github.com/rust-lang/rustlings) course is a good way to learn the basic syntax. Flashing uses [`picotool`](https://github.com/raspberrypi/picotool), and the RP2350 build target is installed with `rustup target add thumbv8m.main-none-eabihf`.

**Wi-Fi credentials** are read at build time as `WIFI_SSID` and `WIFI_PASSWORD` environment values (cargo's `[env]` section). Keep them out of version control, for example in your user-level `~/.cargo/config.toml`.

Logging uses the `log` crate over USB serial, not a debug probe. If the board reboots after a crash, the panic message is printed at the next boot as `LAST PANIC: ...`.

## Repository layout

```
Crux/
├── crates/
│   ├── crux-pico/   Pico 2 W application: sensor drivers, Embassy tasks, Wi-Fi, USB logging
│   └── nav-core/    hardware-independent math: psychrometric/air-data calculations today,
│                    IMU and magnetometer processing being added
├── load_pico.sh     loads the firmware onto the Pico
└── display.sh       shows the live output in the terminal
```

Inside `crux-pico`, `src/drivers/` holds one init function per device and `src/tasks/` holds one Embassy task per sensor or service.

The rule that keeps this reusable: **`crux-pico` depends on `nav-core`, never the other way around.** Nothing in that crate should know about a pin, a bus, or the Embassy runtime.

## How to run

With a Pico 2 W in BOOTSEL mode (hold the BOOTSEL button while plugging in USB), run `./load_pico.sh` to load the Pico, then `./display.sh` to show the live output in the terminal. The serial port re-enumerates after every flash, so reconnect any terminal you have open.

## Future plans

- Fuse the IMU, magnetometer, and barometer into attitude, heading, and altitude estimates in `nav-core`, and compare several filter algorithms.
- Add the GPS and the display, then an on-screen attitude and heading demo.
- View the barometer data remotely and compare the calculated values against WeatherKit.
- Reuse `nav-core` in future projects, including other hardware and vehicles such as a drone.

## Resources

No tutorials were followed for this, but many frameworks and examples were used:

- [Rust by Example](https://doc.rust-lang.org/rust-by-example/index.html), the [Rust Book](https://doc.rust-lang.org/book/title-page.html), and [std](https://doc.rust-lang.org/std/index.html)
- [libm](https://docs.rs/libm/latest/libm/)
- [embassy](https://github.com/embassy-rs/embassy), which is huge
- [rp-pico2w-examples](https://github.com/ninjasource/rp-pico2w-examples) for Wi-Fi
- Sensor driver crates: [bme280](https://docs.rs/bme280/latest/bme280/), [mpu6050-nostd](https://docs.rs/mpu6050-nostd/latest/mpu6050_nostd/), and [qmc5883p](https://docs.rs/qmc5883p/latest/qmc5883p/) (earlier versions used [bme280_rs](https://docs.rs/bme280-rs/latest/bme280_rs/))

## References

Background on what embedded navigation systems are and where they are used:

- Honeywell, [Embedded GPS/INS](https://www.honeywellaerospace.com/us/en/products-and-services/products/navigation-and-sensors/navigation-systems/embedded-gps-ins): an aircraft-grade example listing outputs such as position, velocity, attitude, heading, altitude, and body rates
- SkyMEMS, [What is an embedded GPS inertial navigation system?](https://www.skymems.com/what-is-embedded-gps-inertial-navigation-system/): an overview of inertial navigation and its application areas (a vendor page, so treat its claims as marketing)
- Harley-Davidson, [Embedded Navigation](https://www.harley-davidson.com/us/en/shop/harley-davidson-embedded-navigation/p/76100034): navigation built into a production vehicle

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Disclaimer

Crux is an experimental project built with consumer-grade sensors. Its output is not certified and may be wrong, delayed, or unavailable. Do not rely on it to navigate, to control a vehicle, or for any purpose where failure could cause harm.
