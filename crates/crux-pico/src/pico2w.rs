/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * board.rs
 *
 * Description:
 * Pico 2 W pin map and interrupt bindings. Every pin and
 * peripheral assignment lives here, grouped per device.
 *
 *   BME280    SPI0   SCK GP18  MOSI GP19  MISO GP16  CS GP17
 *   MPU6050   I2C0   SDA GP4   SCL GP5
 *   QMC5883P  I2C1   SDA GP14  SCL GP15
 *   CYW43     PIO0 + DMA_CH0, GP23 / GP24 / GP25 / GP29 (reserved)
 **************************************************************/

/* Public Crates */
use embassy_rp::{
    Peri, Peripherals, bind_interrupts,
    dma::InterruptHandler as DmaHandler,
    i2c::InterruptHandler as I2cHandler,
    peripherals::{
        DMA_CH0, I2C0, I2C1, PIN_4, PIN_5, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19,
        PIN_23, PIN_24, PIN_25, PIN_29, PIO0, SPI0, USB
    },
    pio::InterruptHandler as PioHanlder,
    usb::InterruptHandler as UsbHandler,
};

/* Interrupt Handlers */
bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => PioHanlder<PIO0>;
    DMA_IRQ_0 => DmaHandler<DMA_CH0>;
    I2C0_IRQ => I2cHandler<I2C0>;
    I2C1_IRQ => I2cHandler<I2C1>;
    USBCTRL_IRQ =>UsbHandler<USB>;
});

/* Resources, grouped per device */
pub struct Cyw43Resources {
    pub pwr: Peri<'static, PIN_23>,
    pub cs: Peri<'static, PIN_25>,
    pub pio: Peri<'static, PIO0>,
    pub dio: Peri<'static, PIN_24>,
    pub clk: Peri<'static, PIN_29>,
    pub dma: Peri<'static, DMA_CH0>,
}

pub struct Bme280Resources {
    pub spi: Peri<'static, SPI0>,
    pub sck: Peri<'static, PIN_18>,
    pub sdi: Peri<'static, PIN_19>,
    pub sdo: Peri<'static, PIN_16>,
    pub cs: Peri<'static, PIN_17>,
}

pub struct Mpu6050Resources {
    pub i2c: Peri<'static, I2C0>,
    pub scl: Peri<'static, PIN_5>,
    pub sda: Peri<'static, PIN_4>,
}

pub struct Qmc5883pResources {
    pub i2c: Peri<'static, I2C1>,
    pub scl: Peri<'static, PIN_15>,
    pub sda: Peri<'static, PIN_14>,
}

/* Crux implimentation of resources */
pub struct Pico2w {
    pub usb: Peri<'static, USB>,
    pub cyw43: Cyw43Resources,
    pub bme280: Bme280Resources,
    pub mpu6050: Mpu6050Resources,
    pub qmc5883p: Qmc5883pResources,
}

impl Pico2w {
    pub fn new(p: Peripherals) -> Self {
        Self {
            usb: p.USB,
            cyw43: Cyw43Resources {
                pwr: p.PIN_23,
                 cs: p.PIN_25,
                pio: p.PIO0,
                dio: p.PIN_24,
                clk: p.PIN_29,
                dma: p.DMA_CH0,
            },
            bme280: Bme280Resources {
                spi: p.SPI0,
                sck: p.PIN_18,
                sdi: p.PIN_19,
                sdo: p.PIN_16,
                 cs: p.PIN_17,
            },
            mpu6050: Mpu6050Resources {
                i2c: p.I2C0,
                scl: p.PIN_5,
                sda: p.PIN_4,
            },
            qmc5883p: Qmc5883pResources {
                i2c: p.I2C1,
                scl: p.PIN_15,
                sda: p.PIN_14,
            },
        }
    }
}
