/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * Crux
 *
 * FILE:
 * net_task.rs
 *
 * Description:
 * Task that processes Wi-Fi connection
 **************************************************************/

/* Public Crates */
use cyw43::NetDriver;
use embassy_net::Runner;

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}
