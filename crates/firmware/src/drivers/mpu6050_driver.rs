/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * mpu6050_driver.rs
 *
 * Description:
 * Driver for MPU 6050 sensor
 **************************************************************/

/* Public Crates */
use embassy_rp::{
    Peri,
    i2c::{Config as I2cConfig, I2c},
    peripherals::{I2C0, PIN_4, PIN_5},
};
use embassy_time::Timer;
use log::{error, info};
use mpu6050_nostd::{AccelRange, DlpfConfig, GyroRange, Mpu6050, registers::ADDR_AD0_LOW};

/* Local Types */
pub type MPU6050I2c = Mpu6050<I2c<'static, I2C0, embassy_rp::i2c::Blocking>>;

pub async fn init_mpu6050(
    i2c_periph: Peri<'static, I2C0>,
    scl: Peri<'static, PIN_5>,
    sda: Peri<'static, PIN_4>,
) -> Result<MPU6050I2c, ()> {
    /* create i2c bus */
    let mut i2c = I2c::new_blocking(i2c_periph, scl, sda, I2cConfig::default());

    /* read power bit */
    let mut pwr = [0u8];
    let _ = i2c.blocking_write_read(0x68u8, &[0x6B], &mut pwr);
    info!("PWR_MGMT_1 = {:#04x}", pwr[0]);

    /* wake up sensor if it is asleep (0x40) */
    if pwr[0] == 0x40 {
        info!("MPU 6050 asleep, waking up");
        if let Err(e) = i2c.blocking_write(0x68u8, &[0x6B, 0x01]) {
            error!("wake failed: {:?}", e);
        }
        Timer::after_millis(100).await;
    }

    info!("Initialize MPU 6050");
    let mut mpu6050 = Mpu6050::new(i2c, ADDR_AD0_LOW);
    if let Err(e) = mpu6050.init() {
        error!("failed to init mpu6050: {:?}", e);
        return Err(());
    }
    Timer::after_millis(100).await;

    /* set the accelerometer range */
    if let Err(e) = mpu6050.set_accel_range(AccelRange::G16) {
        error!("Failed to set accelerometer range: {:?}", e);
        return Err(());
    }
    Timer::after_millis(50).await;

    /* set the gyroscope range */
    if let Err(e) = mpu6050.set_gyro_range(GyroRange::Dps2000) {
        error!("Failed to set gyroscope range: {:?}", e);
        return Err(());
    }
    Timer::after_millis(50).await;

    /* set the digital low pass filter */
    if let Err(e) = mpu6050.set_dlpf(DlpfConfig::Hz44) {
        error!("Failed to set digital low pass filter: {:?}", e);
        return Err(());
    }
    Timer::after_millis(50).await;

    Ok(mpu6050)
}
