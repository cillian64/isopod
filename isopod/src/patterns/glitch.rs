//! "glitch" pattern: randomly illuminate segments of the spines in random
//! bright colours

// I find clippy's style to be less clear
#![allow(clippy::needless_range_loop)]

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::led::{SPINES, LEDS_PER_SPINE};

use rand::Rng;

/// Average length of glitching periods in seconds
const GLITCH_LEN: f32 = 0.6;

/// Average length of gaps between glitching periods, in seconds
const GAP_LEN: f32 = 0.8;

/// Number of frames per second.  Changing this doesn't actually affect the
/// FPS, that is controlled by frame-skipping, but this value is used to
/// convert average-time-length values into per-frame probabilities
const FPS: f32 = 60.0;

// The basic "glitch" logic is that at the start of each glitching period we
// choose a number of segments of LEDs which will be glitching and assign each
// a colour.  During glitching, each glitch segment has a certain probability
// of turning on or off in each frame.  Maybe also a small probability of
// changing colour.

/// Maximum number of glitching segments - the actual value is randomly chosen
/// between 0 and this number
const MAX_NUM_SEGMENTS: usize = 30;

/// Probability of a glitching segment turning on in each frame
const GLITCH_P_ON: f32 = 0.1;

/// Probability of a glitching segment turning off in each frame
const GLITCH_P_OFF: f32 = 0.1;

/// Probability of a glitch changing colour
const GLITCH_P_COLOUR: f32 = 0.03;

/// Length of segments to turn on
const GLITCH_SEG_LEN_MIN: usize = 15;

/// Length of segments to turn on
const GLITCH_SEG_LEN_MAX: usize = 40;

/// Represents a contiguous set of LEDs on one spine which are glitching on
/// and off
struct Segment {
    pub spine: usize,
    pub start: usize,
    pub end: usize,
    pub colour: [u8; 3],
    pub on: bool,
}

pub struct Glitch {
    leds: LedUpdate,

    /// The pattern cycles through periods of "glitching" and periods of
    /// all LEDs being off
    glitching: bool,

    /// The segments which are glitching during this glitch period
    segments: Vec<Segment>,

    rng: rand::rngs::ThreadRng,
}

impl Glitch {
    pub const NAME: &'static str = "glitch";
}

impl Pattern for Glitch {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
            },
            glitching: false,
            rng: rand::thread_rng(),
            segments: vec![],
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        if self.glitching {
            // Consider stopping glitching next frame
            if self.rng.gen::<f32>() < 1.0 / (FPS * GLITCH_LEN) {
                self.glitching = false;
                self.segments.clear();
            }

            // Render glitches:
            // Consider turning some segments on or off
            for segment in self.segments.iter_mut() {
                if !segment.on && self.rng.gen::<f32>() < GLITCH_P_ON {
                    // Turn this segment on
                    segment.on = true;

                    // Consider changing colour
                    if self.rng.gen::<f32>() < GLITCH_P_COLOUR {
                        segment.colour = [
                            self.rng.gen::<u8>(),
                            self.rng.gen::<u8>(),
                            self.rng.gen::<u8>(),
                        ];
                    }

                    let spine = &mut self.leds.spines[segment.spine];
                    for led in segment.start..segment.end {
                        spine[led] = segment.colour;
                    }
                } else if segment.on && self.rng.gen::<f32>() < GLITCH_P_OFF {
                    // Turn this segment off
                    segment.on = false;
                    let spine = &mut self.leds.spines[segment.spine];
                    for led in segment.start..segment.end {
                        spine[led] = [0, 0, 0];
                    }
                }
            }
        } else {
            // Consider starting glitching next frame
            if self.rng.gen::<f32>() < 1.0 / (FPS * GAP_LEN) {
                self.glitching = true;

                // Choose some segments to glitch.  These might overlap - if
                // so then ones later in the list will overlap ones earlier
                // in the list
                for _ in 0..self.rng.gen_range(0..MAX_NUM_SEGMENTS) {
                    // Pick a random spine
                    let spine = self.rng.gen_range(0..SPINES);

                    // Choose the length of segment
                    let len: usize = self.rng.gen_range(GLITCH_SEG_LEN_MIN..GLITCH_SEG_LEN_MAX);

                    // Pick an LED range to turn on
                    // Start of range, inclusive
                    let start: usize = self.rng.gen_range(0..(LEDS_PER_SPINE - len));

                    // End of range, inclusive
                    let end: usize = start + len;

                    // Pick a random colour for this segment
                    let colour = [
                        self.rng.gen::<u8>(),
                        self.rng.gen::<u8>(),
                        self.rng.gen::<u8>(),
                    ];

                    self.segments.push(Segment {
                        spine,
                        start,
                        end,
                        colour,
                        on: false,
                    });
                }
            }

            // Turn all LEDs off
            for spine in self.leds.spines.iter_mut() {
                for led in spine.iter_mut() {
                    *led = [0, 0, 0];
                }
            }
        }

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
