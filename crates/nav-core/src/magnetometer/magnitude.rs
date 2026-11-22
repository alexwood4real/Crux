/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * nav-core
 *
 * FILE:
 * magnitude.rs
 *
 * Description:
 * Earth's magnetic field is composed of seven parameters.
 * Given the Veritical Intensity (z) along with the North (x) and Easy (y)
 * components, we are able to calculate the Total Intensity (F) which is known
 * as the magnitude. 
 *
 * Formula:
 * Magnitude of a Vector
 *
 * F = sqrt(x^2 + y^2 + z^2)
 *
 * Sources:
 * https://www.ncei.noaa.gov/products/geomagnetism-frequently-asked-questions
 **************************************************************/
 use libm::{powf, sqrtf};

 pub fn calculate_magnitude(f: [f32; 3]) -> f32 {
    let [x, y, z] = f;

    let mag: f32 = sqrtf( powf(x, 2.0) + powf(y, 2.0) + powf(z, 2.0));

    mag
 }