use std::fmt;

const GRAVITY: f32 = 1.0;
// TODO: Maybe have the tube length have some hidden slots where the core would be
const TUBE_LEN: usize = 118;
const NUM_BEANS: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Bean {
    // Between 0 and 117 inclusive
    position: usize,

    // Units, pixels/second
    velocity: f32,
}

impl Bean {
    /// Apply a physics time-step to the Bean:
    /// - Apply gravitational acceleration to the bean according to `angle`,
    ///   which is the angle between the bean tube and vertical, in radians.
    /// - Adjust the bean's position based on its velocity
    /// - If the bean reaches the pixel `position_stop` then don't move it any
    ///   further and set its velocity to 0 - the bean has landed
    ///
    ///
    fn step(&mut self, angle: f32, position_stop: usize) {
        // Work out the acceleration component parallel to the bean-tube axis
        let accel_component = GRAVITY * f32::cos(angle);

        // Apply the acceleration
        self.velocity += accel_component;

        // TODO: friction or terminal velocity?

        // Apply the movement and end-stop
        //if self.position > position_stop {
            // eprintln!("\n bean pos {} stop {}", self.position, position_stop);
        //}
        // assert!(self.position <= position_stop);
        let new_position = (self.position as f32 + self.velocity) as usize;
        // eprintln!("Bean accel_comp {} vel {}, moving from {} to {}", accel_component, self.velocity, self.position, usize::min(new_position, position_stop));

        // Which direction the stop acts depends on the angle
        if accel_component >= 0.0 {
            self.position = usize::min(new_position, position_stop);
        } else {
            self.position = usize::max(new_position, position_stop);
        }

        if self.position == position_stop {
            self.velocity = 0.0;
        }
        // assert!(self.position <= position_stop);
    }
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
                position: i,
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

        let mut bean_iter = self.beans.iter().peekable();
        while let Some(bean) = bean_iter.next() {
            // Check the beans are in-order and non-overlapping
            if let Some(next_bean) = bean_iter.peek() {
                assert!(next_bean.position > bean.position);
            }

            // Check the bean position is valid
            #[allow(unused_comparisons)] {
                assert!(bean.position >= 0);
            }
            assert!(bean.position < TUBE_LEN);
        }

    }

    /// Apply a physics step, where `angle`, is the angle between the bean
    /// tube and vertical, in radians.
    fn step(&mut self, angle: f32) {
        self.sanity_check();

        // To enable good bean sliding, we need to step the beans in order
        // starting from whichever is the "bottom" end of the tube.
        // Arbitrarily define angle 0.0 as having slot 0 at the top

        if angle > std::f32::consts::FRAC_PI_2 || angle < -std::f32::consts::FRAC_PI_2 {
            // Bean tube is "inverted", process from start of tube first
            let mut position_stop = 0usize;
            for bean in self.beans.iter_mut() {
                // eprintln!("Processing bean with stop {}", position_stop);
                bean.step(angle, position_stop);
                position_stop = bean.position + 1;
                // eprintln!("Bean ended at {} so moving stop to {}", bean.position, position_stop);
                assert!(bean.position < TUBE_LEN);
            }
        } else {
            // Bean tube is "upright", process from end of tube first
            let mut position_stop = TUBE_LEN - 1;
            for (idx, bean) in self.beans.iter_mut().enumerate().rev() {
                // eprintln!("Processing bean with stop {}", position_stop);
                bean.step(angle, position_stop);

                position_stop = if bean.position == 0 {
                    // Avoid underflows here.  This should only happen on the
                    // final bean so the position_stop isn't used anyway.
                    assert!(idx == 0);
                    0
                } else {
                    bean.position - 1
                };


                // eprintln!("Bean ended at {} so moving stop to {}", bean.position, position_stop);
                assert!(bean.position < TUBE_LEN);
            }
        }

        self.sanity_check();
    }

    /// Is there a bean at the given position?
    fn bean_at_pos(&self, position: usize) -> bool {
        // TODO: use the builtin vec binary search
        for bean in self.beans.iter() {
            if bean.position == position {
                return true;
            }
        }
        false
    }
}

impl fmt::Display for Beans {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for slot in 0..TUBE_LEN {
            if self.bean_at_pos(slot) {
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
