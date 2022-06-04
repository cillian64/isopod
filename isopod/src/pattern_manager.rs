//! The PatternManager struct contains the logic to transition between
//! patterns and to choose an appropriate pattern based on the current
//! movement and orientation

use crate::common_structs::{GpsFix, ImuReadings, LedUpdate};
use crate::motion_sensor::MotionSensor;
use crate::patterns::{beans::Beans, pattern_by_name, Pattern, JUKEBOX};
use crate::SETTINGS;
use rand::Rng;

/// State machine for the pattern manager.  Some of the states have an associated pattern
/// which is the one currently selected for playback.  The pattern can't change without
/// changing state via the Transition state (which does not have an associated pattern)
enum PatternManagerState {
    /// Currently stationary, with pattern based on orientation.  The value is the
    /// pattern currently selected.
    Stationary(Box<dyn Pattern>),

    /// Currently in movement.  The first value is the movement pattern.  We store the
    /// movement pattern directly rather than as a Pattern trait so we can use some
    /// extended methods (e.g. frames_since_last_change).  The second value is a count
    /// of the number of consequtive frames where all bean-tubes have been stacked.
    Movement(Beans, usize),

    /// Transitioning between patterns.  We don't use an external pattern in this state,
    /// we just mutate the final state left behind by whichever pattern was playing last.
    /// This state choses which state will come next based on movement during the
    /// transition time.  The values are the LED state during the transition pattern,
    /// and the frame-counter timing the transition, which is initialised to 0 when
    /// the transition begins.
    Transition(LedUpdate, usize),

    /// This pattern is an island, it is chosen on start-up if the `static_pattern`
    /// option is set in settings.toml.  If it is not chosen on start-up it will
    /// never be transitioned to.  The value is the static pattern which we will
    /// play back forever.
    Static(Box<dyn Pattern>),

    /// Similar to Static, but cycles through all the patterns (except beans) on a
    /// fixed timer and ignores movement.  The first value is the current pattern,
    /// the second is the number of frames spent in the current pattern.
    Jukebox(Box<dyn Pattern>, usize),

    /// Transition between jukebox patterns.  Values are:
    /// * The LEDs in the final frame of the pattern before transition
    /// * Frame count in the transition
    JukeboxTransition(LedUpdate, usize)
}

/// Decides which patterns to play back and does transitions between them.
/// Changes patterns based on movement of the isopod
pub struct PatternManager {
    /// The current state machine position. Also stores the pattern currently being
    /// played back (unless we are in transition state)
    state: PatternManagerState,

    motion: MotionSensor,

    /// If we decide a state change is required, which state should be selected
    /// at the next frame.  State changes are deferred one frame for lifetime
    /// reasons so that we can return an LedUpdate reference with lifetime tied
    /// to the current pattern.
    next_state: Option<PatternManagerState>,
}

impl Default for PatternManager {
    fn default() -> Self {
        Self {
            state: PatternManagerState::Transition(LedUpdate::default(), 0),
            motion: MotionSensor::new(),
            next_state: None,
        }
    }
}

impl PatternManager {
    /// Make a new pattern manager
    pub fn new() -> Self {
        match SETTINGS.get::<String>("static_pattern") {
            // If the user has selected a static pattern, then select the static_pattern
            // state which will persist forever.
            Ok(desired_pattern) => {
                if desired_pattern == "jukebox" {
                    println!("Loading jukebox mode.");
                    Self {
                        state: {
                            let pattern = JUKEBOX[0]();
                            PatternManagerState::Jukebox(pattern, 0)
                        },
                        ..PatternManager::default()
                    }
                } else {
                    println!("Loading static pattern {}", &desired_pattern);
                    let pattern = pattern_by_name(&desired_pattern)
                        .unwrap_or_else(|| panic!("Unknown pattern {}", &desired_pattern))(
                    );
                    Self {
                        state: PatternManagerState::Static(pattern),
                        ..PatternManager::default()
                    }
                }
            }
            // Otherwise, load into the transition pattern
            Err(_) => {
                println!("No static pattern specified, going to transition");
                PatternManager::default()
            }
        }
    }

    /// Monitor the GPS and IMU readings to decide which pattern should
    /// currently be in playback.  Transition between patterns where
    /// necessary.  Run a step of whichever pattern is currently selected
    /// and return an updated set of LED states.
    pub fn step(&mut self, gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate {
        self.motion.push(*imu);
        let shock_detected = self.motion.detect_fast_movement();
        let creep_detected = self.motion.detect_slow_movement();

        // If a state change was deferred from the last step, apply it now
        if let Some(next_state) = self.next_state.take() {
            self.state = next_state;
        }

        // Determine the next state, and the current LEDs state
        let led_state = match &mut self.state {
            PatternManagerState::Stationary(pattern) => {
                let is_sleep = pattern.is_sleep();
                let led_state = pattern.step(gps, imu);
                if shock_detected {
                    println!("PatternManager: Stationary detected shock so going to transition");
                    self.next_state = Some(PatternManagerState::Transition(led_state.clone(), 0));
                } else if creep_detected {
                    println!("PatternManager: Stationary detected creep so going to transition");
                    self.next_state = Some(PatternManagerState::Transition(led_state.clone(), 0));
                }

                // If we're in a stationary pattern *other than sleep*, then
                // after a certain timeout go to the sleep pattern
                if !is_sleep && self.motion.sleep_timeout() {
                    println!("PatternManager: Sleep timeout, so going to transition");
                    self.next_state = Some(PatternManagerState::Transition(led_state.clone(), 0))
                }
                led_state
            }

            PatternManagerState::Movement(pattern, stacked_frames) => {
                if pattern.all_stacked() {
                    *stacked_frames += 1;
                } else {
                    *stacked_frames = 0;
                }

                let led_state = pattern.step(gps, imu);

                if *stacked_frames >= crate::motion_sensor::MOVEMENT_PATTERN_TIMEOUT - 1 {
                    println!("PatternManager: No movement detected so going to transition");
                    self.next_state = Some(PatternManagerState::Transition(led_state.clone(), 0));
                }
                led_state
            }

            PatternManagerState::Transition(led_state, frame_count) => {
                // Transition out by sliding towards the center
                for spine in led_state.spines.iter_mut() {
                    let mut led_iter = spine.iter_mut().peekable();
                    while let Some(led) = led_iter.next() {
                        *led = if let Some(next_led) = led_iter.peek() {
                            **next_led
                        } else {
                            [0, 0, 0]
                        };
                    }
                }

                // If we're at the end of the transition, then decide where to go next
                if *frame_count == 60 {
                    if self.motion.sleep_timeout() {
                        println!("PatternManager: Transitioning to sleep");
                        let pattern = pattern_by_name("sleep").unwrap()();
                        self.next_state = Some(PatternManagerState::Stationary(pattern));
                    } else if shock_detected || creep_detected {
                        println!("PatternManager: Transitioning to movement");
                        self.next_state =
                            Some(PatternManagerState::Movement(Beans::new_direct(), 0));
                    } else {
                        println!("PatternManager: Transitioning to stationary");
                        let pattern =
                            select_stationary_pattern(self.motion.get_latest().unwrap_or_default());
                        self.next_state = Some(PatternManagerState::Stationary(pattern));
                    }
                }

                *frame_count += 1;

                led_state
            }

            PatternManagerState::Static(pattern) => pattern.step(gps, imu),

            PatternManagerState::Jukebox(pattern, frames_in_current) => {
                let led_state = pattern.step(gps, imu);

                if *frames_in_current > 3600 {
                    // Enough time in the current pattern, move to the next one
                    self.next_state = Some(PatternManagerState::JukeboxTransition(led_state.clone(), 0));
                } else {
                    *frames_in_current += 1;
                }

                led_state
            }

            PatternManagerState::JukeboxTransition(led_state, frame_count) => {
                // Transition out by sliding towards the center
                for spine in led_state.spines.iter_mut() {
                    let mut led_iter = spine.iter_mut().peekable();
                    while let Some(led) = led_iter.next() {
                        *led = if let Some(next_led) = led_iter.peek() {
                            **next_led
                        } else {
                            [0, 0, 0]
                        };
                    }
                }

                // If we're at the end of the transition, then decide where to go next
                if *frame_count == 60 {
                    let mut rng = rand::thread_rng();
                    let next_pattern_idx: usize = rng.gen_range(0..JUKEBOX.len());
                    let next_pattern = JUKEBOX[next_pattern_idx]();
                    println!("Jukebox: transitioning to {}", next_pattern.get_name());
                    self.next_state = Some(PatternManagerState::Jukebox(next_pattern, 0));
                }

                *frame_count += 1;

                led_state
            }
        };

        led_state
    }
}

/// Work out our orientation from the average IMU readings and use this to
/// select a pattern from the playlist
fn select_stationary_pattern(imu_average: ImuReadings) -> Box<dyn Pattern> {
    // We can simply segment our orientation into 8 solid-angle zones by
    // looking at the sign of each of the X, Y, and Z axes.  The 8 zones are
    // separated by the X, Y, and Z planes of the accelerometer.  These won't
    // necessarily align with flat faces of the outer icosphere, but given
    // there are many more than 8 faces, hopefully this won't matter.
    let x_sign = imu_average.xa >= 0.0;
    let y_sign = imu_average.ya >= 0.0;
    let z_sign = imu_average.za >= 0.0;
    let pattern = match (x_sign, y_sign, z_sign) {
        (false, false, false) => pattern_by_name("zoom"),
        (false, false, true) => pattern_by_name("starfield"),
        (false, true, false) => pattern_by_name("colourfield"),
        (false, true, true) => pattern_by_name("glitch"),
        (true, false, false) => pattern_by_name("colour_wipes"),
        (true, false, true) => pattern_by_name("wormholes"),
        (true, true, false) => pattern_by_name("sparkles"),
        // Currently we only have 7 unique patterns, so repeat one:
        (true, true, true) => pattern_by_name("colour_wipes"),
    };
    // Instantiate the pattern. Should never be None assuming I can type.
    pattern.unwrap()()
}
