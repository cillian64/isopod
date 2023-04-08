//! Each opposing spine-pair is a tube containing a number of "beans" (or
//! Smarties?) which slide back and forth depending on the orientation of
//! that spine-pair relative to gravity (or the sum acceleration vector the
//! Isopod experiences)
//!
//! This file defines the actual 3d pattern, the bean physics simulation is
//! done in bean_sim.rs

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::geometry;
use crate::patterns::Pattern;
use crate::{LEDS_PER_SPINE, SPINES};

mod bean_sim;

use bean_sim::{BeanTube, TUBE_LEN};

pub struct Beans {
    // Cache this to save allocations even though we overwrite all the LEDs
    leds: LedUpdate,

    // For the software sim, just let our gravity vector wander around
    #[cfg(not(feature = "hardware"))]
    a: f32,
    #[cfg(not(feature = "hardware"))]
    b: f32,
    #[cfg(not(feature = "hardware"))]
    c: f32,

    // Used to slide out beans from the centre at the start
    start_timer: usize,

    // We have 6 bean tubes, one for each opposing pair of spines
    bean_tubes: Vec<BeanTube>,
}

impl Beans {
    pub const NAME: &'static str = "beans";

    /// Make a new Beans pattern object directly, not using the Pattern trait wrapper
    pub fn new_direct() -> Beans {
        let mut bean_tubes = vec![];
        for _ in 0..(SPINES / 2) {
            bean_tubes.push(BeanTube::new());
        }

        Self {
            leds: LedUpdate::default(),
            #[cfg(not(feature = "hardware"))]
            a: 0.0,
            #[cfg(not(feature = "hardware"))]
            b: 0.0,
            #[cfg(not(feature = "hardware"))]
            c: 0.0,
            bean_tubes,
            start_timer: 0,
        }
    }

    /// Are all of our bean tubes stacked at one end or the other
    pub fn all_stacked(&self) -> bool {
        self.bean_tubes.iter().all(|tube| tube.is_stacked())
    }
}

impl Pattern for Beans {
    fn new() -> Box<dyn Pattern> {
        Box::new(Beans::new_direct())
    }

    #[allow(unused_variables)]
    fn step(&mut self, _gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate {
        // When we are in the start time, ignore everything and just render
        // the beans coming out from the centre.  The length of the start time
        // depends on the number of beans
        if self.start_timer < (bean_sim::NUM_BEANS / 2) * 4 {
            for spine in self.leds.spines.iter_mut() {
                for (idx, led) in spine.iter_mut().enumerate() {
                    *led = if idx <= self.start_timer / 4 {
                        [255, 255, 255]
                    } else {
                        [0, 0, 0]
                    };
                }
            }

            self.start_timer += 1;
            return &self.leds;
        }

        // Get the acceleration vector either from the hardware, or if we're
        // doing software sim then fake it.  For hardware, invert the
        // acceleration vector because we want the force applied to the beans,
        // not the acceleration they experience.  Also, scale up accelerometer
        // acceleration a bit to make it more responsive.
        #[cfg(feature = "hardware")]
        let gravity = imu.accel_vector().scale(-5.0);
        #[cfg(not(feature = "hardware"))]
        let gravity = geometry::UnitVector3d::from_angles(self.a, self.b, self.c)
            .as_vector3d()
            .scale(9.81);

        // According to geometry::SPINE_DIRECTIONS, the opposing pairs are:
        // 0 and 3
        // 1 and 2
        // 4 and 7
        // 5 and 6
        // 8 and 11
        // 9 and 10

        for (bean_tube_idx, bean_tube) in self.bean_tubes.iter_mut().enumerate() {
            // Work out which opposing pair of spines corresponds to this bean-tube
            let (spine1, spine2) = match bean_tube_idx {
                0 => (0, 3),
                1 => (1, 2),
                2 => (4, 7),
                3 => (5, 6),
                4 => (8, 11),
                5 => (9, 10),
                _ => panic!("Only know how to deal with 6 bean-tubes."),
            };

            // Work out the angle from gravity and apply a physics step
            let bean_tube_direction = geometry::SPINE_DIRECTIONS[spine1].as_vector3d();
            let acceleration = geometry::dot(&gravity, bean_tube_direction);
            bean_tube.step(acceleration);

            // Illuminate LEDs appropriately
            for (idx, led) in self.leds.spines[spine1].iter_mut().enumerate() {
                *led = bean_tube.get_colour(LEDS_PER_SPINE - 1 - idx);
            }
            for (idx, led) in self.leds.spines[spine2].iter_mut().enumerate() {
                *led = bean_tube.get_colour(TUBE_LEN - 1 - (LEDS_PER_SPINE - 1 - idx));
            }
        }

        #[cfg(not(feature = "hardware"))]
        {
            self.b += std::f32::consts::PI / 60.0;
        }

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
