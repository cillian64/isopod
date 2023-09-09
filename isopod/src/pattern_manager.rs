//! The PatternManager struct contains the logic to transition between
//! patterns and to choose an appropriate pattern based on the current
//! movement and orientation

use crate::common_structs::{GpsFix, ImuReadings, LedUpdate};
use crate::patterns::{pattern_by_name, Pattern, colourwipes::ColourWipes};
use crate::control_server::CONTROLS;

/// State machine for the pattern manager.  Some of the states have an associated pattern
/// which is the one currently selected for playback.  The pattern can't change without
/// changing state via the Transition state (which does not have an associated pattern)
enum PatternManagerState {
    /// Similar to Static, but cycles through all the patterns (except beans) on a
    /// fixed timer and ignores movement.  The first value is the current pattern,
    /// the second is the number of frames spent in the current pattern.
    Jukebox(Box<dyn Pattern>),

    /// Transition between jukebox patterns.  Values are:
    /// * The LEDs in the final frame of the pattern before transition
    /// * Frame count in the transition
    JukeboxTransition(LedUpdate, usize),
}

/// Decides which patterns to play back and does transitions between them.
/// Changes patterns based on movement of the isopod
pub struct PatternManager {
    /// The current state machine position. Also stores the pattern currently being
    /// played back (unless we are in transition state)
    state: PatternManagerState,

    /// If we decide a state change is required, which state should be selected
    /// at the next frame.  State changes are deferred one frame for lifetime
    /// reasons so that we can return an LedUpdate reference with lifetime tied
    /// to the current pattern.
    next_state: Option<PatternManagerState>,
}

impl Default for PatternManager {
    fn default() -> Self {
        Self {
            state: PatternManagerState::JukeboxTransition(LedUpdate::default(), 0),
            next_state: None,
        }
    }
}

impl PatternManager {
    /// Make a new pattern manager
    pub fn new() -> Self {
        let pattern_name = CONTROLS.read().unwrap().pattern.clone();
        // Load pattern if possible, but if not found then just use
        // colour_wipes as default.
        let pattern = match pattern_by_name(&pattern_name) {
            Some(x) => x(),
            None => ColourWipes::new(),
        };

        Self {
            state: PatternManagerState::Jukebox(pattern),
            ..PatternManager::default()
        }
    }

    /// Transition between patterns where
    /// necessary.  Run a step of whichever pattern is currently selected
    /// and return an updated set of LED states.
    pub fn step(&mut self, gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate {
        // If a state change was deferred from the last step, apply it now
        if let Some(next_state) = self.next_state.take() {
            self.state = next_state;
        }

        // Determine the next state, and the current LEDs state
        let led_state = match &mut self.state {
            PatternManagerState::Jukebox(pattern) => {
                // Do this before step() because borrows
                let old_pattern_name = pattern.get_name();

                let led_state = pattern.step(gps, imu);

                // Check if a pattern change is needed:
                let new_pattern_name = CONTROLS.read().unwrap().pattern.clone();
                if new_pattern_name != old_pattern_name {
                    // Pattern has changed, go to transitition
                    self.next_state = Some(PatternManagerState::JukeboxTransition(led_state.clone(), 0));
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
                    let next_pattern_name = CONTROLS.read().unwrap().pattern.clone();

                    // Load pattern if possible, but if not found then just use
                    // colour_wipes as default.
                    let next_pattern = match pattern_by_name(&next_pattern_name) {
                        Some(x) => x(),
                        None => ColourWipes::new(),
                    };

                    println!("Jukebox: transitioning to {}", next_pattern.get_name());
                    self.next_state = Some(PatternManagerState::Jukebox(next_pattern));
                }

                *frame_count += 1;

                led_state
            }
        };

        led_state
    }
}
