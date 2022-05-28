//! This module defines the physics simulation used for the "beans" pattern.

use std::fmt;

/// New physics algorithm:
/// - Each bean stores floating point position and velocity
/// - Reduce gravity and do smaller time-steps - do 10 or 100 steps per frame
/// - Step:
///   - For each bean, in arbitrary order
///     - Apply acceleration to its velocity
///     - Calculate next_position = position + velocity
///     - Depending on the sign of next_position look left or right to find
///       either the tube end or next bean which blocks us and limit
///       next_position as appropriate.  Set velocity to 0 if we collide

#[allow(unused)]
const GRAVITY: f32 = 1.0;
// TODO: Maybe have the tube length have some hidden slots where the core would be
pub const TUBE_LEN: usize = 118;
pub const NUM_BEANS: usize = 41;

// The Pi can keep up something like 1000 physics steps per second.  At 60fps,
// 10 steps/frame works out at 600 steps/sec which works well.
const STEPS_PER_FRAME: usize = 10;
const DT: f32 = 1.0 / (STEPS_PER_FRAME as f32);

/// Represents a tube with some beans in it which can slide back and forth.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bean {
    /// Between 0 and 117.0 inclusive
    position: f32,

    /// Units, pixels/second
    velocity: f32,

    /// The colour of the bean...
    colour: [u8; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeanTube {
    beans: Vec<Bean>,
}

impl BeanTube {
    pub fn new() -> Self {
        let mut beans = Vec::<Bean>::new();
        let first_bean_pos = (TUBE_LEN as f32) / 2.0 - (NUM_BEANS as f32) / 2.0;
        for i in 0..NUM_BEANS {
            beans.push(Bean {
                position: first_bean_pos + (i as f32),
                velocity: 0.0,
                colour: [255, 255, 255],
            });
        }
        BeanTube { beans }
    }

    /// Check that:
    /// - There are the correct number of beans
    /// - The beans are still in-order
    /// - The beans obey the Pauli exclusion principle
    /// - The bean positions are in-range
    fn sanity_check(&self) {
        // Check we have the right number of beans
        assert!(self.beans.len() == NUM_BEANS);
        assert!(self.beans.len() < TUBE_LEN);

        let epsilon = 0.1; // allowable error

        let mut bean_iter = self.beans.iter().peekable();
        while let Some(bean) = bean_iter.next() {
            // Check the beans are in-order and non-overlapping
            if let Some(next_bean) = bean_iter.peek() {
                assert!(next_bean.position - bean.position > 1.0 - epsilon);
            }

            // Check the bean position is valid
            assert!(bean.position > -epsilon);
            assert!(bean.position < TUBE_LEN as f32 - 1.0 + epsilon);
        }
    }

    /// Apply a physics step, where `angle`, is the angle between the bean
    /// tube and vertical, in radians.
    #[allow(unused)]
    pub fn step_angle(&mut self, angle: f32) {
        let acceleration = GRAVITY * f32::cos(angle);
        self.step(acceleration);
    }

    /// Apply a physics step, where `accel` is the total acceleration
    /// component in the direction of this bean-tube.
    pub fn step(&mut self, acceleration: f32) {
        for _ in 0..STEPS_PER_FRAME {
            self.sub_step(acceleration / 100.0);
        }
    }

    /// Each "step" is actually a number of physics steps.  This is the physics
    /// step which gets repeated multiple times per frame
    fn sub_step(&mut self, acceleration: f32) {
        self.sanity_check();

        // Add some randomness to each velocity, it makes it look better.
        // Work out the magnitude of randomness to apply
        let fuzz_magnitude = f32::abs(acceleration);

        // For each bean, apply acceleration and calculate next_position
        // ignoring collisions
        for i in 0..NUM_BEANS {
            let fuzz = fuzz_magnitude * (2.0 * rand::random::<f32>() - 1.0);
            self.beans[i].velocity += (acceleration + fuzz) * DT;
            // Hypothetical next position, subject to change due to collisions
            let mut next_position = self.beans[i].position + self.beans[i].velocity * DT;

            // Determine which way to look for collisions.  Don't bother
            // looking if velocity is 0.0
            if self.beans[i].velocity > 0.0 {
                // Look right
                if let Some(right_bean) = self.bean_to_right(i) {
                    // Collide with right_bean
                    if right_bean.position - next_position < 1.0 {
                        next_position = right_bean.position - 1.0;
                        // If it's moving right, match its velocity.  Otherwise stop
                        if right_bean.velocity > 0.0 {
                            self.beans[i].velocity = right_bean.velocity;
                        } else {
                            self.beans[i].velocity = 0.0;
                        }
                    }
                } else {
                    // Collide with right edge of tube
                    if TUBE_LEN as f32 - 1.0 - next_position < 1.0 {
                        next_position = TUBE_LEN as f32 - 1.0;
                        self.beans[i].velocity = 0.0;
                    }
                }
            } else if self.beans[i].velocity < 0.0 {
                // Look left
                if let Some(left_bean) = self.bean_to_left(i) {
                    // Collide with left_bean
                    if next_position - left_bean.position < 1.0 {
                        next_position = left_bean.position + 1.0;
                        // If it's moving left, match its velocity.  Otherwise stop
                        if left_bean.velocity < 0.0 {
                            self.beans[i].velocity = left_bean.velocity;
                        } else {
                            self.beans[i].velocity = 0.0;
                        }
                    }
                } else {
                    // Collide with left edge of tube
                    if next_position < 0.0 {
                        next_position = 0.0;
                        self.beans[i].velocity = 0.0;
                    }
                }
            }
            self.beans[i].position = next_position;
        }
        self.sanity_check();
    }

    /// Is there a bean at the given position? If so, retrieve it
    fn bean_at_pos(&self, position: usize) -> Option<Bean> {
        // TODO: use the builtin vec binary search
        for bean in self.beans.iter() {
            if f32::round(bean.position) as usize == position {
                return Some(*bean);
            }
        }
        None
    }

    /// Get the bean immediately to the left of the one with the supplied
    /// index, or return None if there are no beans to its left.
    fn bean_to_left(&self, i: usize) -> Option<Bean> {
        assert!(i < NUM_BEANS);
        if i == 0 {
            None
        } else {
            Some(self.beans[i - 1])
        }
    }

    /// Get the bean immediately to the left of the one with the supplied
    /// index, or return None if there are no beans to its left.
    fn bean_to_right(&self, i: usize) -> Option<Bean> {
        assert!(i < NUM_BEANS);
        if i == NUM_BEANS - 1 {
            None
        } else {
            Some(self.beans[i + 1])
        }
    }

    /// Each bean has a colour.  If there is a bean at the requested position
    /// then return its colour.  Otherwise, return black ([0, 0, 0]).
    #[allow(unused)]
    pub fn get_colour(&self, i: usize) -> [u8; 3] {
        match self.bean_at_pos(i) {
            Some(bean) => bean.colour,
            None => [0, 0, 0],
        }
    }

    /// Return true if all the beans are stacked at one end, false otherwise
    #[allow(unused)]
    pub fn is_stacked(&self) -> bool {
        // Bean 0 sits at 0.0, bean 1 at 1.0, etc.
        let stacked_left = self.beans.last().unwrap().position < (NUM_BEANS as f32) - 0.5;

        // Bean N-1 sits at T-1 (where T is tube_len), N-2 sits at T-2, etc.
        // So bean N-N sits at T-N
        let stacked_right = self.beans[0].position > (TUBE_LEN - NUM_BEANS) as f32 - 0.5;

        stacked_right || stacked_left
    }
}

impl fmt::Display for BeanTube {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for slot in 0..TUBE_LEN {
            if self.bean_at_pos(slot).is_some() {
                write!(f, "#")?;
            } else {
                write!(f, " ")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}
