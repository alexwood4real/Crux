/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * nav-core
 *
 * FILE:
 * inclination.rs
 *
 * Description:
 * The inclination, or dip, is the angle made with the horizonal
 * by Earth's magnetic field lines. The value varies based on location.
 * Positive: magnetic field of earth is pointing towards earth
 * Negative: magnetic field of earth is pointing upwards
 * On a compass, this would be the needle of a vertically held compass
 *
 * Formula:
 * Inclination Formula
 *
 * tan(I) = Bz / Bh => I = arctan(Bz / Bh)
 *
 * Sources:
 * https://www.ncei.noaa.gov/products/geomagnetism-frequently-asked-questions
 **************************************************************/
 use libm::atan2f;

 pub fn calculate_inclination(z: f32, hmag: f32) -> f32 {
    atan2f(z, hmag)
 }