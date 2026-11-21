/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * main.rs
 *
 * Description:
 * Initializes sensors, buses, and spawns tasks
 **************************************************************/

#![no_std]
#![no_main]

/* Public Crates */
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    dma::InterruptHandler as DmaHandler,
    i2c::InterruptHandler as I2cHandler,
    peripherals::{DMA_CH0, I2C0, I2C1, PIO0, USB},
    pio::InterruptHandler as PioHanlder,
    usb::{Driver as UsbDriver, InterruptHandler as UsbHandler},
};
use embassy_time::{Duration, Timer};
use log::{error, info};

/* Modules */
mod panic;

/* Tasks */
mod tasks;
use tasks::{
    bme280_task::bme280_task, logger_task::logger_task, mpu6050_task::mpu6050_task,
    qmc5883p_task::qmc5883p_task,
};

/* Drivers */
mod drivers;
use drivers::{
    bme280_driver::init_bme280, cyw43_driver::init_cyw43, mpu6050_driver::init_mpu6050,
    qmc5883p_driver::init_qmc5883p,
};

/* Interrupt Handlers */
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioHanlder<PIO0>;
    DMA_IRQ_0 => DmaHandler<DMA_CH0>;
    I2C0_IRQ => I2cHandler<I2C0>;
    I2C1_IRQ => I2cHandler<I2C1>;
    USBCTRL_IRQ =>UsbHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /* init RP2350 peripherals */
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    info!("Configure logger");
    let driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver).unwrap());
    Timer::after(Duration::from_secs(3)).await;
    panic::report_last_panic();

    info!("Configuring CYW43 Wi-Fi chip");
    init_cyw43(
        spawner, p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0, Irqs,
    )
    .await;

    info!("Configure BME 280 sensor");
    let bme280_spi = p.SPI0;
    let bme280_sck = p.PIN_18;
    let bme280_sdi = p.PIN_19;
    let bme280_sdo = p.PIN_16;
    let bme280_cs = p.PIN_17;
    let bme280 = match init_bme280(bme280_spi, bme280_sck, bme280_sdi, bme280_sdo, bme280_cs) {
        Ok(b) => b,
        Err(_) => loop {
            error!("BME 280 init failed, check wiring");
            Timer::after_secs(1).await;
        },
    };

    info!("Configure MPU 6050 sensor");
    let mpu6050_i2c = p.I2C0;
    let mpu6050_sda = p.PIN_4;
    let mpu6050_scl = p.PIN_5;
    let mpu6050 = match init_mpu6050(mpu6050_i2c, mpu6050_scl, mpu6050_sda).await {
        Ok(m) => m,
        Err(_) => loop {
            error!("MPU 6050 init failed, check wiring");
            Timer::after_secs(1).await;
        },
    };

    info!("Configure QMC 5883P sensor");
    let qmc5883p_i2c = p.I2C1;
    let qmc5883p_sda = p.PIN_14;
    let qmc5883p_scl = p.PIN_15;
    let qmc5883p = match init_qmc5883p(qmc5883p_i2c, qmc5883p_scl, qmc5883p_sda, Irqs).await {
        Ok(q) => q,
        Err(_) => loop {
            error!("QMC 5883P init failed, check wiring");
            Timer::after_secs(1).await;
        },
    };

    info!("Spawn sensor tasks");
    spawner.spawn(bme280_task(bme280).unwrap());
    spawner.spawn(mpu6050_task(mpu6050).unwrap());
    spawner.spawn(qmc5883p_task(qmc5883p).unwrap());
}

/* Metadata */
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Crux"),
    embassy_rp::binary_info::rp_program_description!(c"This program runs Crux, a Navigation Subsystem."),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// End of file
