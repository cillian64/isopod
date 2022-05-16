//! The PatternManager struct contains the logic to transition between
//! patterns and to choose an appropriate pattern based on the current
//! movement and orientation

use crate::common_structs::{GpsFix, ImuReadings, LedUpdate};
use crate::patterns::{pattern_by_name, Pattern};
use crate::SETTINGS;

/// State machine for the pattern manager.  Some of the states have an associated pattern
/// which is the one currently selected for playback.  The pattern can't change without
/// changing state via the Transition state (which does not have an associated pattern)
enum PatternManagerState {
    /// Currently stationary, with pattern based on orientation.  The value is the
    /// pattern currently selected.
    Stationary(Box<dyn Pattern>),

    /// Currently in movement.  The value is the movement pattern (which will probably
    /// always be the same one)
    Movement(Box<dyn Pattern>),

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
}

/// Length of the moving average used for movement detection, affects sensitivity
const MOVING_AVERAGE_LEN: usize = 120; // 2 seconds, at 60fps

/// Threshold for movement detection
const MOVEMENT_THRESH: f32 = 1.0; // TODO: adjust this value

/// Decides which patterns to play back and does transitions between them.
/// Changes patterns based on movement of the isopod
pub struct PatternManager {
    /// The current state machine position. Also stores the pattern currently being
    /// played back (unless we are in transition state)
    state: PatternManagerState,

    /// Circular buffer, used to work out a moving average of the accelerometer
    /// and gyro readings
    moving_average_buffer: Vec<ImuReadings>,

    /// Next position to write to in the circulate buffer.
    moving_average_ptr: usize,

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
            moving_average_buffer: vec![],
            moving_average_ptr: 0,
            next_state: None,
        }
    }
}

impl PatternManager {
    /// Make a new pattern manager
    pub fn new() -> Self {
        match SETTINGS.get::<String>("static_pattern".into()) {
            // If the user has selected a static pattern, then select the static_pattern
            // state which will persist forever.
            Ok(desired_pattern) => {
                println!("Loading static pattern {}", &desired_pattern);
                let pattern = pattern_by_name(&desired_pattern)
                    .unwrap_or_else(|| panic!("Unknown pattern {}", &desired_pattern))(
                );
                Self {
                    state: PatternManagerState::Static(pattern),
                    ..PatternManager::default()
                }
            }
            // Otherwise, load into the transition pattern
            Err(_) => {
                println!("No static pattern specified, going to transition");
                PatternManager::default()
            }
        }
    }

    /// Add the new reading to the moving average buffer, then calculate and
    /// return the current moving average of the readings in the buffer.
    fn update_moving_average(&mut self, imu: &ImuReadings) -> ImuReadings {
        // Update the moving average buffer
        if self.moving_average_buffer.len() < MOVING_AVERAGE_LEN {
            // Before the circular buffer is full, just append to it.  Ignore the pointer
            self.moving_average_buffer.push(*imu);
        } else {
            self.moving_average_buffer[self.moving_average_ptr] = *imu;
            self.moving_average_ptr = (self.moving_average_ptr + 1) % MOVING_AVERAGE_LEN;
        }

        // Calculate the current moving average
        self.moving_average_buffer
            .iter()
            .cloned()
            .sum::<ImuReadings>()
            / (MOVING_AVERAGE_LEN as f32)
    }

    /// Monitor the GPS and IMU readings to decide which pattern should
    /// currently be in playback.  Transition between patterns where
    /// necessary.  Run a step of whichever pattern is currently selected
    /// and return an updated set of LED states.
    pub fn step(&mut self, gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate {
        let imu_average = self.update_moving_average(imu);

        // If a state change was deferred from the last step, apply it now
        if let Some(next_state) = self.next_state.take() {
            self.state = next_state;
        }

        // Determine the next state, and the current LEDs state
        let led_state = match &mut self.state {
            PatternManagerState::Stationary(pattern) => {
                let led_state = pattern.step(gps, imu);
                if imu_average.gyro_magnitude() > MOVEMENT_THRESH {
                    self.next_state = Some(PatternManagerState::Transition(led_state.clone(), 0));
                }
                led_state
            }

            PatternManagerState::Movement(pattern) => {
                let led_state = pattern.step(gps, imu);
                if imu_average.gyro_magnitude() < MOVEMENT_THRESH {
                    self.next_state = Some(PatternManagerState::Transition(led_state.clone(), 0));
                }
                led_state
            }

            PatternManagerState::Transition(led_state, frame_count) => {
                // Fade out the LEDs a bit:
                for spine in led_state.spines.iter_mut() {
                    for led in spine.iter_mut() {
                        led[0] = u8::max(1, led[0]) - 1;
                        led[1] = u8::max(1, led[1]) - 1;
                        led[2] = u8::max(1, led[2]) - 1;
                    }
                }

                // If we're at the end of the transition, then decide where to go next
                if *frame_count == 119 {
                    if imu_average.gyro_magnitude() > MOVEMENT_THRESH {
                        let pattern = pattern_by_name("beans").unwrap()();
                        self.next_state = Some(PatternManagerState::Movement(pattern));
                    } else {
                        let pattern = select_stationary_pattern(&imu_average);
                        self.next_state = Some(PatternManagerState::Stationary(pattern));
                    }
                }

                *frame_count += 1;

                led_state
            }

            PatternManagerState::Static(pattern) => pattern.step(gps, imu),
        };

        led_state
    }
}

/// Work out our orientation from the average IMU readings and use this to
/// select a pattern from the playlist
fn select_stationary_pattern(imu_average: &ImuReadings) -> Box<dyn Pattern> {
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
        (false, false, true)  => pattern_by_name("starfield"),
        (false, true, false)  => pattern_by_name("colourfield"),
        (false, true, true)   => pattern_by_name("glitch"),
        (true, false, false)  => pattern_by_name("colour_wipes"),
        // Currently we only have 5 unique patterns, so repeat some:
        (true, false, true)   => pattern_by_name("colourfield"),
        (true, true, false)   => pattern_by_name("glitch"),
        (true, true, true)    => pattern_by_name("colour_wipes"),
    };
    // Instantiate the pattern. Should never be None assuming I can type.
    pattern.unwrap()()
}
