/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * nav-core
 *
 * FILE:
 * horizontal_magnitude.rs
 *
 * Description:
 * Earth's magnetic field is composed of seven parameters.
 * Given the North (x) and East (y) components we are able 
 * to calculate the Horizontal Intensity (H) 
 * 
 * Formula:
 * Magnitude of a Vector
 *
 * H = sqrt(x^2 + y^2)
 *
 * Sources:
 * https://www.ncei.noaa.gov/products/geomagnetism-frequently-asked-questions
 **************************************************************/
 use libm::{powf, sqrtf};

 pub fn calculate_horizontal_magnitude(f: [f32; 2]) -> f32 {
    let [x, y] = f;

    sqrtf(powf(x, 2.0) + powf(y, 2.0))
 }