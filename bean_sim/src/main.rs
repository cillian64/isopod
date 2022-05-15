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


const GRAVITY: f32 = 1.0;
// TODO: Maybe have the tube length have some hidden slots where the core would be
const TUBE_LEN: usize = 118;
const NUM_BEANS: usize = 40;

const STEPS_PER_FRAME: usize = 100;
const DT: f32 = 1.0 / (STEPS_PER_FRAME as f32);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bean {
    /// Between 0 and 117.0 inclusive
    position: f32,

    /// Units, pixels/second
    velocity: f32,
}

#[derive(Clone, Debug, PartialEq)]
struct Beans {
    beans: Vec<Bean>,
}

impl Beans {
    pub fn new() -> Self {
        let mut beans = Vec::<Bean>::new();
        for i in 0..NUM_BEANS {
            beans.push(Bean {
                position: i as f32,
                velocity: 0.0,
            });
        }
        Beans { beans }
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
    pub fn step(&mut self, angle: f32) {
        for _ in 0..STEPS_PER_FRAME {
            self.sub_step(angle);
        }
    }

    /// Each "step" is actually a number of physics steps.  This is the physics
    /// step which gets repeated multiple times per frame
    fn sub_step(&mut self, angle: f32) {
        self.sanity_check();

        let acceleration = GRAVITY * f32::cos(angle);

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
}

impl fmt::Display for Beans {
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


fn main() {
    println!("Hello, world!");

    let mut beans = Beans::new();
    let mut counter: usize = 0;
    let mut angle_flippr = false;
    loop {
        println!("                 {}", beans);

        if counter % 30 == 0 {
            angle_flippr = !angle_flippr;
            //println!("FLIP");
        }

        let angle = if angle_flippr {
            std::f32::consts::FRAC_PI_2 + std::f32::consts::PI / 20.0
        } else {
            std::f32::consts::FRAC_PI_2 - std::f32::consts::PI / 20.0
        };
        beans.step(angle);

        counter += 1;
        std::thread::sleep(std::time::Duration::from_millis(1000 / 10));
    }

}
