/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * PFD
 *
 * FILE:
 * main.rs
 *
 * Description:
 * Blinking LED ensures Wi-Fi connection and data collection
 * Using BME280, MPU6050, QMC5883P sensors and GPS Module
 **************************************************************/

#![no_std]
#![no_main]

/* Crates */
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use core::ptr::{addr_of, addr_of_mut};
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    dma::InterruptHandler as DmaHandler,
    i2c::{InterruptHandler as I2cHandler, I2c, Config as I2cConfig},
    peripherals::{DMA_CH0, I2C0, I2C1, PIO0, USB},
    pio::InterruptHandler as PioHanlder,
    usb::{Driver as UsbDriver, InterruptHandler as UsbHandler},
};
use embassy_time::{Duration, Timer};
use log::{error, info};

/* QMC 5883P Crates*/
use qmc5883p::{Range, Mode,Qmc5883p, Qmc5883PConfig, OutputDataRate, OverSampleRate, OverSampleRatio1};

/* Tasks */
mod tasks;
use tasks::{bme280_task::bme280_task, logger_task::logger_task, mpu6050_task::mpu6050_task};

/* Drivers */
mod drivers;
use drivers::{bme280_driver::init_bme280, cyw43_driver::init_cyw43, mpu6050_driver::init_mpu6050};

/* Interrupt Handlers */
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioHanlder<PIO0>;
    DMA_IRQ_0 => DmaHandler<DMA_CH0>;
    I2C0_IRQ => I2cHandler<I2C0>;
    I2C1_IRQ => I2cHandler<I2C1>;
    USBCTRL_IRQ =>UsbHandler<USB>;
});

/* Constants */
const PANIC_MAGIC: u32 = 0x5041_4E43;

/* Debugging */
#[repr(C)]
struct PanicRecord {
    magic: u32,
    len: u32,
    msg: [u8; 240],
}

// .uninit is not zeroed at boot, so this survives a reset
#[unsafe(link_section = ".uninit.panic_record")]
static mut PANIC_RECORD: MaybeUninit<PanicRecord> = MaybeUninit::uninit();

struct Cursor<'a> {
    buf: &'a mut [u8],
    pos: usize,
}
impl Write for Cursor<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let n = s.len().min(self.buf.len() - self.pos);
        self.buf[self.pos..self.pos + n].copy_from_slice(&s.as_bytes()[..n]);
        self.pos += n;
        Ok(())
    }
}

fn record(args: core::fmt::Arguments) -> ! {
    unsafe {
        let rec = addr_of_mut!(PANIC_RECORD) as *mut PanicRecord;
        let mut c = Cursor {
            buf: &mut *addr_of_mut!((*rec).msg),
            pos: 0,
        };
        let _ = c.write_fmt(args);
        (*rec).len = c.pos as u32;
        (*rec).magic = PANIC_MAGIC;
    }
    cortex_m::peripheral::SCB::sys_reset()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    record(format_args!("{}", info))
}

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    record(format_args!(
        "HardFault pc={:#010x} lr={:#010x}",
        ef.pc(),
        ef.lr()
    ))
}

fn report_last_panic() {
    unsafe {
        let rec = addr_of_mut!(PANIC_RECORD) as *mut PanicRecord;
        if core::ptr::read_volatile(addr_of!((*rec).magic)) == PANIC_MAGIC {
            let len = (core::ptr::read_volatile(addr_of!((*rec).len)) as usize).min(240);
            let msg = &*addr_of!((*rec).msg);
            if let Ok(s) = core::str::from_utf8(&msg[..len]) {
                log::error!("LAST PANIC: {}", s);
            }
            core::ptr::write_volatile(addr_of_mut!((*rec).magic), 0);
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("PFD start");

    /* Init RP2350 peripherals */
    let p: embassy_rp::Peripherals = embassy_rp::init(Default::default());

    /* Initialize Logger */
    info!("Configure logger");
    let driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver).unwrap());
    Timer::after(Duration::from_secs(3)).await;
    report_last_panic();

    /* Configure CYW43 chip */
    info!("Configuring CYW43 Wi-Fi chip");
    init_cyw43(
        spawner, p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0, Irqs,
    )
    .await;

    /* Configure BME 280 sensor */
    info!("Configure BME 280 sensor");
    let bme280_spi = p.SPI0;
    let bme280_sck = p.PIN_18;
    let bme280_sdi = p.PIN_19;
    let bme280_sdo = p.PIN_16;
    let bme280_cs = p.PIN_17;
    let _bme280 = match init_bme280(bme280_spi, bme280_sck, bme280_sdi, bme280_sdo, bme280_cs) {
        Ok(b) => b,
        Err(_) => loop {
            error!("BME 280 init failed, check wiring");
            Timer::after_secs(1).await;
        },
    };

    /* Configure MPU 6050 Sensor */
    info!("Configure MPU 6050 sensor");
    let mpu6050_i2c = p.I2C0;
    let mpu6050_sda = p.PIN_4;
    let mpu6050_scl = p.PIN_5;
    let _mpu6050 = match init_mpu6050(mpu6050_i2c, mpu6050_scl, mpu6050_sda).await {
        Ok(m) => m,
        Err(_) => loop {
            error!("MPU 6050 init failed, check wiring");
            Timer::after_secs(1).await;
        }
    };

    /* Main: Set up I2C1 for QMC 5883P */
    let qmc5883p_i2c = p.I2C1;
    let qmc5883p_sda = p.PIN_14;
    let qmc5883p_scl = p.PIN_15;
    let i2c1 = I2c::new_async(qmc5883p_i2c, qmc5883p_scl, qmc5883p_sda, Irqs, I2cConfig::default());

    /* Driver: Configure QMC 5883P Sensor */
    let qmc5883p_config = Qmc5883PConfig::default()
        .with_mode(Mode::Continuous)
        .with_odr(OutputDataRate::Hz100)
        .with_range(Range::Gauss8)
        .with_osr1(OverSampleRatio1::Ratio4)
        .with_osr2(OverSampleRate::Rate4);

    /* Driver: Initialize QMC 5883P Sensor */
    let mut qmc5883p = Qmc5883p::new(i2c1);
    if let Err(e) = qmc5883p.init(qmc5883p_config).await {
        error!("Failed to init QMC 5883p sensor: {:?}", e);
        return;
    } Timer::after_millis(50).await;

    /* Task: Read QMC 5883P Data */
    loop {
        match qmc5883p.read_x_y_z().await {
            Ok([x, y, z]) => info!("fields: x ={} y={} z={}", x, y, z),
            Err(e) => error!("Failed to read QMC5883P fields: {:?}", e),
        }
        Timer::after_millis(100).await;

        // match qmc5883p.read_magnitude().await {
        //     Ok(m) => info!("magnitude = {}", m),
        //     Err(e) => error!("failed to magnitude: {:?}", e),
        // }
        // Timer::after_millis(100).await;
        // TODO: their read_magnitude function is very simple:
        /*
            pub async fn read_magnitude(&mut self) -> Result<f32, QmcError<E>> {
            let data = self.read_x_y_z().await?;
            let x = data[0] as f32;
            let y = data[1] as f32;
            let z = data[2] as f32;

            let magnitude = (x * x + y * y + z * z).sqrt();
            Ok(magnitude)
        }
         */
    }


    // info!("Spawn sensor tasks");
    // spawner.spawn(bme280_task(bme280).unwrap());
    // spawner.spawn(mpu6050_task(mpu6050).unwrap());
}

/* Metadata */
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"PFD"),
    embassy_rp::binary_info::rp_program_description!(c"This program runs the Primary Flight Deck."),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// End of file
