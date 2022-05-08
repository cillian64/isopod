//! "Colour wipes", stolen from Bitstream.  White bands move outwards from the
//! root of each spine, starting at random times, leaving behind a trail of
//! random colour

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::{LEDS_PER_SPINE, SPINES};
use color_space::{Hsv, Rgb};
use rand::Rng;

/// How fast the wipes move in pixels per frame
const WIPE_SPEED: f32 = 0.5;

/// How long is the white band, in pixels
const BAND_LEN: usize = 5;

/// The colour of the head band leading each wipe
const BAND_COLOUR: [u8; 3] = [255, 255, 255];

/// How much should the tail pixels decay by each frame
const DECAY: u8 = 2;

pub struct ColourWipes {
    leds: LedUpdate,
    rng: rand::rngs::ThreadRng,

    /// Wipes currently in progress.  They can overlap, and are rendered in
    /// the order they appear in the vec
    wipes: Vec<Wipe>,

    /// Frame counter, used for timing new wipes
    frame_counter: u64,
}

/// Represents a single colour "wipe", a white band which moves along leaving
/// a trail of colour behind it
struct Wipe {
    /// Which spine does this wipe live on
    spine: usize,

    /// The position of the "head" of the band, starts at 0 and finishes past
    /// LEDS_PER_SPINE.  Expressed as a float to make slow movement easier
    position: f32,

    /// What colour does the wipe leave behind it
    colour: [u8; 3],
}

impl ColourWipes {
    pub const NAME: &'static str = "colour_wipes";

    fn random_colour(&mut self) -> [u8; 3] {
        let hue = self.rng.gen::<f64>() * 360.0;
        let saturation = self.rng.gen::<f64>() * 0.5 + 0.5;
        let value = 1.0f64;

        let hsv = Hsv::new(hue, saturation, value);
        let rgb = Rgb::from(hsv);
        [rgb.r as u8, rgb.g as u8, rgb.b as u8]
    }
}

impl Pattern for ColourWipes {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
            },
            rng: rand::thread_rng(),
            wipes: vec![],
            frame_counter: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        // First, apply decay to all non-white pixels
        for spine in self.leds.spines.iter_mut() {
            for led in spine.iter_mut() {
                // White pixels are the head, don't decay them
                if DECAY > 0 && *led != [255, 255, 255] {
                    *led = [
                        std::cmp::max(led[0], DECAY) - DECAY,
                        std::cmp::max(led[1], DECAY) - DECAY,
                        std::cmp::max(led[2], DECAY) - DECAY,
                    ];
                }
            }
        }

        // At each step, move all of the wipes along and render them in
        // order
        for wipe in self.wipes.iter_mut() {
            let old_pos = wipe.position;
            wipe.position += WIPE_SPEED;

            // If the wipe position has moved by 1 pixel or more since the
            // last frame then draw one leading pixel of head and one pixel of
            // tail trailing behind the head.  This code assumes that the
            // speed is less than one pixel per frame
            static_assertions::const_assert!(WIPE_SPEED <= 1.0);

            if (wipe.position as usize) != (old_pos as usize) {
                // Band has moved a pixel so we need to do some drawing

                // Draw the first pixel of the head
                let head_position = wipe.position as i32;
                if head_position >= 0 && head_position < (LEDS_PER_SPINE as i32) {
                    self.leds.spines[wipe.spine][head_position as usize] = BAND_COLOUR;
                }

                // Draw the first pixel of the tail
                let tail_position = wipe.position as i32 - (BAND_LEN as i32);
                if tail_position >= 0 && tail_position < (LEDS_PER_SPINE as i32) {
                    self.leds.spines[wipe.spine][tail_position as usize] = wipe.colour;
                }
            }
        }

        // Start a new wipe on each spine in turn on a certain period
        let period = 20u64;
        if self.frame_counter % period == 0 {
            let spine = ((self.frame_counter / period) % SPINES as u64) as usize;
            let colour = self.random_colour();
            self.wipes.push(Wipe {
                spine,
                position: -(BAND_LEN as f32),
                colour,
            })
        }

        self.frame_counter += 1;

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
