use embassy_rp::{
    Peri, 
    i2c::{Config as I2cConfig, I2c}, 
    peripherals::{I2C1, PIN_14, PIN_15}
};
use embassy_time::Timer;
use qmc5883p::{Range, Mode, Qmc5883p, Qmc5883PConfig, OutputDataRate, OverSampleRate, OverSampleRatio1};
use log::{error, info};

pub type QMC5883PI2c = Qmc5883p<I2c<'static, I2C1, embassy_rp::i2c::Async>>;

pub async fn init_qmc5883p(
    i2c_periph: Peri<'static, I2C1>,
    scl: Peri<'static, PIN_15>,
    sda: Peri<'static, PIN_14>,
    irq: crate::Irqs
) -> Result<QMC5883PI2c, ()> {
    /* Set up I2C bus */
    let i2c = I2c::new_async(i2c_periph, scl, sda, irq, I2cConfig::default());

    /* Configure QMC 5883P Sensor */
    let qmc5883p_config = Qmc5883PConfig::default()
        .with_mode(Mode::Continuous)
        .with_odr(OutputDataRate::Hz100)
        .with_range(Range::Gauss8)
        .with_osr1(OverSampleRatio1::Ratio4)
        .with_osr2(OverSampleRate::Rate4);

    /* Initialize QMC 5883P Sensor */
    info!("Initialize QMC 5883P");
    let mut qmc5883p = Qmc5883p::new(i2c);
    if let Err(e) = qmc5883p.init(qmc5883p_config).await {
        error!("Failed to init QMC 5883p sensor: {:?}", e);
        return Err(());
    } Timer::after_millis(50).await;

    Ok(qmc5883p)
}