/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * qmc5883_task.rs
 *
 * Description:
 * Task that reads and processes data from QMC 5883P
 **************************************************************/

/* Public Crates */
use embassy_time::Timer;
use log::{error, info};

/* Local Crates */
use crate::drivers::qmc5883p_driver::QMC5883PI2c;

#[embassy_executor::task]
pub async fn qmc5883p_task(mut qmc5883p: QMC5883PI2c) -> ! {
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
}
