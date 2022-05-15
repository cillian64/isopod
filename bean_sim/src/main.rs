use std::fmt;

/// New physics algorithm:
/// - Each bean stores floating point position, velocity, and next_position
/// - Step:
///   - For each bean
///     - Apply acceleration to its velocity
///     - Calculate next_position = position + velocity
///
///   - For each bean, starting with the left-most
///     - If the bean is travelling to the left:
///       - If the bean is the left-most
///         - If next_position is <= the left edge of the tube
///           - Set next_position = the left edge of the tube
///           - Zero its velocity
///       - Else if the bean's next_position is <= that of the bean to the left of it
///         - Set its next_position to that of the bean to the left of it + 1
///         - If the bean to the left of it is travelling left or stationary, set
///           this bean's velocity to that of the bean to the left of it
///         - If the bean to the left is travelling right, zero this bean's velocity
///
///   - For each bean, starting with the right-most
///     - If the bean is travelling to the right:
///       - If the bean is the right-most:
///         - If next_position is >= the right edge of the tube
///           - Set next_position = the right edge of the tube
///           - Zero its velocity
///       - Else if the bean's next_position is >= that of the bean to the right of it
///         - Set its next_position to that of the bean to the right of it - 1
///         - If the bean to the right of it is travelling right or stationary, set
///           this bean's velocity to that of the bean to the right of it
///         - If the bean to the right is travelling left, zero this bean's velocity
///
/// For robustness, don't run any assertions when in production - any issues
/// won't affect rendering, probably won't be noticable, and will probably shake
/// out in the next physics step.
///
///
/// For implementation neatness, implement
/// fn get_left_bean(&self, from: usize) -> Option<Bean>
/// fn get_right_bean(&self, from: usize) -> Option<Bean>



const GRAVITY: f32 = 1.0;
// TODO: Maybe have the tube length have some hidden slots where the core would be
const TUBE_LEN: usize = 118;
const NUM_BEANS: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bean {
    /// Between 0 and 117.0 inclusive
    position: f32,

    /// Units, pixels/second
    velocity: f32,

    /// The desired position for this bean at the end of this physics step.
    /// May be temporarily invalid before collision corrections are applied.
    next_position: f32
}

#[derive(Clone, Debug, PartialEq)]
struct Beans {
    beans: Vec<Bean>,
}

impl Beans {
    fn new() -> Self {
        let mut beans = Vec::<Bean>::new();
        for i in 0..NUM_BEANS {
            beans.push(Bean {
                position: i as f32,
                velocity: 0.0,
                next_position: i as f32,
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
        return;
        // Check we have the right number of beans
        assert!(self.beans.len() == NUM_BEANS);
        assert!(self.beans.len() < TUBE_LEN);

        let mut bean_iter = self.beans.iter().peekable();
        while let Some(bean) = bean_iter.next() {
            // Check the beans are in-order and non-overlapping
            if let Some(next_bean) = bean_iter.peek() {
                assert!(next_bean.position > bean.position);
            }

            // Check the bean position is valid
            #[allow(unused_comparisons)] {
                assert!(bean.position >= 0.0);
            }
            assert!(bean.position <= TUBE_LEN as f32 - 1.0);
        }

    }

    /// Apply a physics step, where `angle`, is the angle between the bean
    /// tube and vertical, in radians.
    fn step(&mut self, angle: f32) {
        self.sanity_check();

        let acceleration = GRAVITY * f32::cos(angle);

        // For each bean, apply acceleration and calculate next_position
        // ignoring collisions
        for bean in self.beans.iter_mut() {
            bean.velocity += acceleration;
            // This is a hypothetical next position in the absence of
            // collisions with other beans and tube ends.
            bean.next_position = bean.position + bean.velocity;
        }

        // Handle collisions for left-moving beans
        for i in 0..(NUM_BEANS - 1) {
            // Exclude any right-moving or stationary beans
            if self.beans[i].velocity >= 0.0 {
                continue;
            }

            if let Some(left_bean) = self.bean_to_left(i) {
                // Handle collisions with left-bean
                let mut bean = &mut self.beans[i];
                if bean.next_position - left_bean.next_position < 1.0 {
                    // We have collided with left-bean: sit to the right of it
                    bean.next_position = left_bean.next_position + 1.0;
                    // If left-bean is moving left, coallesce with it.
                    // Otherwise stop both beans
                    if left_bean.velocity <= 0.0 {
                        // Coallesce the beans
                        bean.velocity = left_bean.velocity;
                    } else {
                        // Stop the bean.  Left-bean is travelling right so
                        // will be stopped later
                        bean.velocity = 0.0;
                    }
                }
            } else {
                // This is the left-most bean, handle collisions with the left
                // end of the tube
                let mut bean = &mut self.beans[i];
                if bean.next_position <= 0.0 {
                    bean.next_position = 0.0;
                }
            }

        }

        // Handle collisions for right-moving beans
        for i in (0..(NUM_BEANS - 1)).rev() {
            // Exclude any left-moving beans
            if self.beans[i].velocity < 0.0 {
                continue;
            }

            if let Some(right_bean) = self.bean_to_right(i) {
                // Handle collisions with right-bean
                let mut bean = &mut self.beans[i];
                if right_bean.next_position - bean.next_position < 1.0 {
                    // We have collided with right-bean: sit to the left of it
                    bean.next_position = right_bean.next_position - 1.0;
                    // If left-bean is moving right, coallesce with it.
                    // Otherwise stop both beans
                    if right_bean.velocity >= 0.0 {
                        // Coallesce the beans
                        bean.velocity = right_bean.velocity;
                    } else {
                        // Stop the bean.  Right-bean is travelling left so
                        // should have already stopped earlier
                        bean.velocity = 0.0;
                    }
                }
            } else {
                // This is the right-most bean, handle collisions with the right
                // end of the tube
                let mut bean = &mut self.beans[i];
                if bean.next_position >= (TUBE_LEN - 1) as f32 {
                    bean.next_position = (TUBE_LEN - 1) as f32;
                }
            }
        }

        // Finally, apply all of the physics steps
        for bean in self.beans.iter_mut() {
            bean.position = bean.next_position;
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
        print!("\r{}", beans);

        if counter % 120 == 0 {
            angle_flippr = !angle_flippr;
            println!("\nFLIP");
        }

        let angle = if angle_flippr {
            std::f32::consts::FRAC_PI_2 + std::f32::consts::PI / 10.0
        } else {
            std::f32::consts::FRAC_PI_2 - std::f32::consts::PI / 10.0
        };
        beans.step(angle);

        counter += 1;
        std::thread::sleep(std::time::Duration::from_millis(1000 / 10));
    }

}
