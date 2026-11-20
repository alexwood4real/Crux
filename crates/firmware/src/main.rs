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
    i2c::{InterruptHandler as I2cHandler, Config as I2cConfig, I2c},
    peripherals::{DMA_CH0, I2C0, PIO0, USB},
    pio::InterruptHandler as PioHanlder,
    usb::{Driver as UsbDriver, InterruptHandler as UsbHandler},
};
use embassy_time::{Duration, Timer};
use log::{error, info};
use mpu6050_nostd::{AccelRange, Mpu6050, registers::ADDR_AD0_LOW};

/* Tasks */
mod tasks;
use tasks::{bme280_task::bme280_task, logger_task::logger_task};

/* Drivers */
mod drivers;
use drivers::{bme280_driver::init_bme280, cyw43_driver::init_cyw43};

/* Interrupt Handlers */
bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => PioHanlder<PIO0>;
    DMA_IRQ_0 => DmaHandler<DMA_CH0>;
    I2C0_IRQ => I2cHandler<I2C0>;
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
    let bme280 = match init_bme280(bme280_spi, bme280_sck, bme280_sdi, bme280_sdo, bme280_cs) {
        Ok(b) => b,
        Err(_) => loop {
            error!("BME 280 init failed, check wiring");
            Timer::after_secs(1).await;
        },
    };

    /* launch sensor tasks */
    info!("Spawn sensor tasks");
    spawner.spawn(bme280_task(bme280).unwrap());

    /* Configure and read MPU 6050 Sensor */
    info!("Configure MPU 6050 sensor");
    let mpu6050_i2c = p.I2C0;
    let mpu6050_sda = p.PIN_4;
    let mpu6050_scl = p.PIN_5;
    let mut i2c = I2c::new_blocking(mpu6050_i2c, mpu6050_scl, mpu6050_sda, I2cConfig::default());
    /* TODO: put in README because this is so useful i did it with the bme as */
    // let mut i2c = I2c::new_blocking(mpu6050_i2c, mpu6050_scl, mpu6050_sda, I2cConfig::default());
    // let mut id = [0u8];
    // match i2c.blocking_write_read(0x68u8, &[0x75], &mut id) {
    //     Ok(()) => info!("MPU WHO_AM_I = {:#04x}", id[0]), // 0x68 expected; clones may differ
    //     Err(e) => error!("no MPU6050 at 0x68: {:?}", e),
    // }

    // 1. Is it asleep? (0x40 = sleep bit set)
    let mut pwr = [0u8];
    let _ = i2c.blocking_write_read(0x68u8, &[0x6B], &mut pwr);
    info!("PWR_MGMT_1 = {:#04x}", pwr[0]);

    // 2. Wake it: clear sleep, use the gyro X clock
    if let Err(e) = i2c.blocking_write(0x68u8, &[0x6B, 0x01]) {
        error!("wake failed: {:?}", e);
    }
    Timer::after_millis(100).await;

    // 3. Raw read: accel X/Y/Z, temp, gyro X/Y/Z (14 bytes from 0x3B)
    let mut raw = [0u8; 14];
    match i2c.blocking_write_read(0x68u8, &[0x3B], &mut raw) {
        Ok(()) => {
            let ax = i16::from_be_bytes([raw[0], raw[1]]);
            let ay = i16::from_be_bytes([raw[2], raw[3]]);
            let az = i16::from_be_bytes([raw[4], raw[5]]);
            info!("raw ax={} ay={} az={}", ax, ay, az);
        }
        Err(e) => error!("raw read failed: {:?}", e),
    }

    /* -------------------- */
    
    let mut mpu6050 = Mpu6050::new(i2c, ADDR_AD0_LOW);
    if let Err(e) = mpu6050.init(){
        error!("failed to init mpu6050: {:?}", e);
        return;
    } Timer::after_millis(100).await;

    if let Err(e) = mpu6050.set_accel_range(AccelRange::G8) {
        error!("Failed to set range: {:?}", e);
        return;
    }Timer::after_millis(50).await;
    
    loop {
    match mpu6050.accel_g() {
        Ok([x, y, z]) => info!("accel g: x={} y={} z={}", x, y, z),
        Err(e) => error!("Failed to read MPU: {:?}", e),
    }
    Timer::after_millis(100).await;
}

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
