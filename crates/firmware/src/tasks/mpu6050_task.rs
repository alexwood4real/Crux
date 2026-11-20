use embassy_time::Timer;
use log::{error, info};

use crate::{drivers::mpu6050_driver::MPU6050I2c};

#[embassy_executor::task]
pub async fn mpu6050_task(mut mpu6050: MPU6050I2c) -> ! {
    /* MPU 6050 task */
    loop {
        match mpu6050.accel_g() {
            Ok([x, y, z]) => info!("accel g: x={} y={} z={}", x, y, z),
            Err(e) => error!("Failed to read allel_g: {:?}", e),
        }
        Timer::after_millis(100).await;

        match mpu6050.gyro_dps() {
            Ok([x, y, z]) => info!("gyro: x={} y={} z={}", x, y, z),
            Err(e) => error!("Failed to read gyro_dps: {:?}", e),
        }
        Timer::after_millis(100).await;
    }
}