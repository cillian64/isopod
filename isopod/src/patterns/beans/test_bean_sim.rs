//! This file defines a standalone console application used for testing the
//! bean_sim physics library.  It can be run by using the following command:
//! cargo run --target=x86_64-unknown-linux-gnu --bin test_bean_sim

mod bean_sim;

fn main() {
    let mut beans = bean_sim::BeanTube::new();
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
        beans.step_angle(angle);

        counter += 1;
        std::thread::sleep(std::time::Duration::from_millis(1000 / 10));
    }

}
