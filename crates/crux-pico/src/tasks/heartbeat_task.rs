/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * heartbeat_task.rs
 *
 * Description:
 * Task that blinks LED on Pico 2 W board
 **************************************************************/

 /* Public Crates */
use embassy_time::{Duration, Timer};
use log::info;

#[embassy_executor::task]
pub async fn heartbeat_task(mut control: cyw43::Control<'static>) -> ! {
    info!("Heartbeat start");
    let delay: Duration = Duration::from_millis(500);

    loop {
        control.gpio_set(0, true).await;
        Timer::after(delay).await;

        control.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}
