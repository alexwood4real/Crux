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
use embassy_rp::usb::Driver as UsbDriver;
use embassy_time::{Duration, Timer};
use log::{error, info};

/* Modules */
mod drivers;
mod panic;
mod pico2w;
mod tasks;

/* Board */
use pico2w::{Pico2w, Irqs};

/* Tasks */
use tasks::{
    bme280_task::bme280_task, logger_task::logger_task, mpu6050_task::mpu6050_task,
    qmc5883p_task::qmc5883p_task,
};

/* Drivers */
use drivers::{
    bme280_driver::init_bme280, cyw43_driver::init_cyw43, mpu6050_driver::init_mpu6050,
    qmc5883p_driver::init_qmc5883p,
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    /* init RP2350 peripherals */
    let pico2w = Pico2w::new(embassy_rp::init(Default::default()));

    info!("Configure logger");
    let driver = UsbDriver::new(pico2w.usb, Irqs);
    spawner.spawn(logger_task(driver).unwrap());
    Timer::after(Duration::from_secs(3)).await;
    panic::report_last_panic();

    info!("Configuring CYW43 Wi-Fi chip");
    let cyw43 = pico2w.cyw43;
    init_cyw43(
        spawner, cyw43.pwr, cyw43.cs, cyw43.pio, cyw43.dio,cyw43.clk,cyw43.dma, Irqs,
    )
    .await;

    /* spawn sensors */
    
    info!("Configure BME 280 sensor");
    let bme280 = pico2w.bme280;
    match init_bme280(bme280.spi, bme280.sck, bme280.sdi, bme280.sdo, bme280.cs) {
        Ok(b) => spawner.spawn(bme280_task(b).unwrap()),
        Err(_) => error!("Failed to init BME 280, check wiring"),
    }

    info!("Configure MPU 6050 sensor");
    let mpu6050 = pico2w.mpu6050;
    match init_mpu6050(mpu6050.i2c, mpu6050.scl, mpu6050.sda).await {
        Ok(m) => spawner.spawn(mpu6050_task(m).unwrap()),
        Err(_) => error!("Failed to init MPU 6050, check wiring"),
    }

    info!("Configure QMC 5883P sensor");
    let qmc5883p = pico2w.qmc5883p;
    match init_qmc5883p(qmc5883p.i2c, qmc5883p.scl, qmc5883p.sda, Irqs).await {
        Ok(q) => spawner.spawn(qmc5883p_task(q).unwrap()),
        Err(_) => error!("Failed to init QMC 5883P, check wiring"),
    }
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
