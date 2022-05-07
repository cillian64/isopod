//! Spin a search-light around a randomly wandering axis

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::geometry;
use crate::patterns::Pattern;
use crate::led::{SPINES, LEDS_PER_SPINE};

pub struct Searchlight {
    // Cache this to save allocations even though we overwrite all the LEDs
    leds: LedUpdate,

    // Our three Euler angles which wander around
    a: f32,
    b: f32,
    c: f32,
}

impl Searchlight {
    pub const NAME: &'static str = "searchlight";
}

impl Pattern for Searchlight {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
            },
            a: 0.0,
            b: 0.0,
            c: 0.0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        // Illuminate the spines within a 45 degree cone of the vector.
        let light_direction = geometry::UnitVector3d::from_angles(self.a, self.b, self.c);

        for (spine_num, spine) in self.leds.spines.iter_mut().enumerate() {
            let spine_direction = &geometry::SPINE_DIRECTIONS[spine_num];
            let angle = geometry::unit_vector_angle_with_dir(spine_direction, &light_direction);

            let colour = if angle > 0.0 {
                // println!("Spine {} direction {:?} angle {} ON",
                //     spine_num, spine_direction, angle);
                [64, 64, 64]
            } else {
                // println!("Spine {} direction {:?} angle {} OFF",
                //     spine_num, spine_direction, angle);
                [0, 0, 0]
            };

            for led in spine.iter_mut() {
                *led = colour;
            }
        }

        self.b += std::f32::consts::PI / 60.0;

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
