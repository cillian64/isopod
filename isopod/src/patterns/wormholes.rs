//! Colourful worms appear from wormholes, travel a short distance, then disappear
//! again

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;

use color_space::{Hsv, Rgb};
use rand::{rngs::ThreadRng, Rng};

const WORMHOLE_MAX_LEN: i32 = 10;
const WORMHOLE_MIN_LEN: i32 = 5;
const WORM_HEAD_FADE_LEN: i32 = 2;
const WORM_TAIL_FADE_LEN: i32 = 10;
const WORM_LEN: i32 = 20;
/// Probability of a new wormhole, per frame
const WORMHOLE_RATE: f32 = 1.0 / 5.0;

/// Generate a fully saturated colour with a random hue
fn random_colour(rng: &mut ThreadRng) -> [u8; 3] {
    let hue = rng.gen::<f64>() * 360.0;
    let saturation = 1.0;
    let value = 1.0f64;

    let hsv = Hsv::new(hue, saturation, value);
    let rgb = Rgb::from(hsv);
    [rgb.r as u8, rgb.g as u8, rgb.b as u8]
}

struct WormHole {
    // Use i32 all over the place to avoid problems with underflows
    /// Which spine is this wormhole on
    spine: usize,

    /// Start position of this wormhole
    start: i32,

    /// End position of this wormhole
    end: i32,

    /// Speed of this worm in inverse units, i.e. 2 means one pixel every other frame
    /// The velocity is signed - the worm can go in either direction!
    velocity: i32,

    /// Position of worm head
    worm_head_pos: i32,

    /// The colour of the body of this worm
    colour: [u8; 3],

    // Ranges between 0 and velocity, used to track fractional head movements
    fractional_head_pos: usize,
}

impl WormHole {
    /// Make a new wormhole with random properties
    pub fn new(existing_wormholes: &[WormHole], rng: &mut ThreadRng) -> Option<Self> {
        // Make 10 attempts to make a new wormhole without overlapping an
        // existing one.  If we still can't do it after that then give up.
        // In theory we should be safe to just keep trying until we find one
        // that works, but limiting the attempts protects against programming
        // mistakes which might make a new wormhole impossible.

        for _ in 0..10 {
            let start: i32 = rng.gen_range(0..54);
            let mut end: i32 = rng.gen_range(0..59);
            end = i32::min(end, start + WORMHOLE_MIN_LEN);
            end = i32::max(end, start + WORMHOLE_MAX_LEN);
            end = i32::min(end, 59);
            let direction = rng.gen_range(0..=1) * 2 - 1;
            // Velocity magnitude 1-3 with random sign
            let velocity = rng.gen_range(1..3) * direction;

            let candidate = WormHole {
                spine: rng.gen_range(0..12),
                start,
                end,
                velocity,
                // TODO: Start a bit earlier for fade-in
                worm_head_pos: if velocity > 0 { start } else { end - 1 },
                colour: random_colour(rng),
                fractional_head_pos: 0,
            };

            if existing_wormholes
                .iter()
                .any(|existing| existing.overlaps(&candidate))
            {
                // Candidate overlaps, try again
                continue;
            }

            // Candidate looks good
            return Some(candidate);
        }
        // No candidate found after 10 attempts, ugh
        None
    }

    fn overlaps(&self, other: &WormHole) -> bool {
        self.spine == other.spine && self.end >= other.start && other.end >= self.start
    }

    /// Apply a time-step to this wormhole
    pub fn step(&mut self) {
        self.fractional_head_pos += 1;
        if self.fractional_head_pos == i32::abs(self.velocity) as usize {
            if self.velocity > 0 {
                self.worm_head_pos += 1;
            } else {
                self.worm_head_pos -= 1;
            }
            self.fractional_head_pos = 0;
        }
    }

    fn fade_colour(colour: [u8; 3], proportion: f32) -> [u8; 3] {
        [
            f32::round(colour[0] as f32 * proportion) as u8,
            f32::round(colour[1] as f32 * proportion) as u8,
            f32::round(colour[2] as f32 * proportion) as u8,
        ]
    }

    /// Render this wormhole onto the LEDs.  Assumes that all LEDs have been
    /// cleared to black before rendering begins.
    pub fn render(&self, leds: &mut LedUpdate) {
        let spine = &mut leds.spines[self.spine];
        for (idx, led) in spine.iter_mut().enumerate() {
            let idx = idx as i32;
            if idx < self.start || idx >= self.end {
                // Not inside the wormhole
                continue;
            }

            // TODO: Deduplicate forward/backward worm code
            // TODO: Deduplicate fade code

            *led = if self.velocity > 0 {
                // "Forwards" worm
                if idx >= self.worm_head_pos + WORM_HEAD_FADE_LEN {
                    // Ahead of the head
                    [0, 0, 0]
                } else if idx > self.worm_head_pos {
                    // In the head fade region
                    // Fade proportion from 0.0 to 1.0
                    let fade_proportion = (WORM_HEAD_FADE_LEN - (idx - self.worm_head_pos)) as f32
                        / WORM_HEAD_FADE_LEN as f32;
                    Self::fade_colour([255, 255, 255], fade_proportion)
                } else if idx == self.worm_head_pos {
                    // Worm head is white
                    [255, 255, 255]
                } else if idx > self.worm_head_pos - WORM_LEN {
                    // Worm body
                    self.colour
                } else if idx >= self.worm_head_pos - WORM_LEN - WORM_TAIL_FADE_LEN {
                    // In the tail fade region
                    // Fade proportion from 0.0 to 1.0
                    let fade_proportion =
                        (idx - (self.worm_head_pos - WORM_LEN - WORM_TAIL_FADE_LEN)) as f32
                            / WORM_TAIL_FADE_LEN as f32;
                    Self::fade_colour(self.colour, fade_proportion)
                } else {
                    // Behind worm body
                    [0, 0, 0]
                }
            } else {
                // "Backwards" worm
                if idx <= self.worm_head_pos - WORM_HEAD_FADE_LEN {
                    // Ahead of the head
                    [0, 0, 0]
                } else if idx < self.worm_head_pos {
                    // In the head fade region
                    // Fade proportion from 0.0 to 1.0
                    let fade_proportion = (WORM_HEAD_FADE_LEN - (self.worm_head_pos - idx)) as f32
                        / WORM_HEAD_FADE_LEN as f32;
                    Self::fade_colour([255, 255, 255], fade_proportion)
                } else if idx == self.worm_head_pos {
                    // Worm head is white
                    [255, 255, 255]
                } else if idx < self.worm_head_pos + WORM_LEN {
                    // Worm body
                    self.colour
                } else if idx <= self.worm_head_pos + WORM_LEN + WORM_TAIL_FADE_LEN {
                    // In the tail fade region
                    // Fade proportion from 0.0 to 1.0
                    let fade_proportion = ((self.worm_head_pos + WORM_LEN + WORM_TAIL_FADE_LEN)
                        - idx) as f32
                        / WORM_TAIL_FADE_LEN as f32;
                    Self::fade_colour(self.colour, fade_proportion)
                } else {
                    // Behind worm body
                    [0, 0, 0]
                }
            }
        }
    }

    /// Return true if this wormhole is finished and should be destroyed
    pub fn finished(&self) -> bool {
        if self.velocity > 0 {
            self.worm_head_pos > self.end + WORM_LEN + WORM_HEAD_FADE_LEN + WORM_TAIL_FADE_LEN
        } else {
            self.worm_head_pos < self.start - WORM_LEN - WORM_HEAD_FADE_LEN - WORM_TAIL_FADE_LEN
        }
    }
}

pub struct WormHoles {
    leds: LedUpdate,
    wormholes: Vec<WormHole>,
    rng: ThreadRng,
}

impl WormHoles {
    pub const NAME: &'static str = "wormholes";
}

impl Pattern for WormHoles {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            wormholes: vec![],
            rng: rand::thread_rng(),
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        // Consider making a new wormhole
        if self.rng.gen::<f32>() < WORMHOLE_RATE {
            // Trying to create a new wormhole might not succeed if we get
            // really unlucky with overlaps.
            if let Some(new_wormhole) = WormHole::new(&self.wormholes, &mut self.rng) {
                self.wormholes.push(new_wormhole);
            }
        }

        // Clear LEDs - we render everything from scratch every frame
        for spine in self.leds.spines.iter_mut() {
            for led in spine.iter_mut() {
                *led = [0, 0, 0];
            }
        }

        // Time-step and render each wormhole
        for wormhole in self.wormholes.iter_mut() {
            wormhole.step();
            wormhole.render(&mut self.leds);
        }

        // Prune any finished wormholes
        self.wormholes.retain(|wormhole| !wormhole.finished());

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
