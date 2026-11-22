/**************************************************************
 * SPDX-License-Identifier: MIT OR Apache-2.0
 * nav-core
 *
 * FILE:
 * magnetometer.rs
 *
 * Description:
 * Handler to process magnetic data in the X, Y, Z directions
 **************************************************************/

 pub mod magnitude;
 pub mod horizontal_magnitude;
 pub mod inclination;

 pub use magnitude::calculate_magnitude;
 pub use horizontal_magnitude::calculate_horizontal_magnitude;
 pub use inclination::calculate_inclination;

 /* holds initial mag reading data */
 pub struct FieldData {
    pub fields: [f32; 3],  /* (x, y, z) */
 }

 /* holds calculated magnetometer data */
 pub struct MagnetometerData {
    pub fields: [f32; 3],               /* Magnetic Fields */
    pub magnitude: f32,                 /* Total Magnetic Field Strength */
    pub horizontal_magnitude: f32,      /* Horizontal Magnetic Field Strength */
    pub inclination: f32,               /* Inclination */
 }

 /* calcualte data from initial field readings */
 impl FieldData {
    pub fn calculate(&self) -> Option<MagnetometerData> {
        /* unwrap field values */
        let [x, y, z]: [f32; 3] = self.fields;

        /* collect data */
        let mag: f32 = calculate_magnitude([x, y, z]);
        let hmag: f32 = calculate_horizontal_magnitude([x, y]);
        let i: f32 = calculate_inclination(z, hmag);

        Some(MagnetometerData { 
            fields: [x, y, z], 
            magnitude: mag,
            horizontal_magnitude: hmag,
            inclination: i,
        })
    }
 }