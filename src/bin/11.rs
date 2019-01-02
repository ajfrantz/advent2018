use failure::Error;
use rayon::prelude::*;

fn calc_power(x: i32, y: i32) -> i32 {
    let rack_id = x + 10;
    let serial = 2866;
    ((rack_id * y + serial) * rack_id / 100) % 10 - 5
}

fn main() -> Result<(), Error> {
    let (best, best_x, best_y, best_l) = (1..301)
        .into_par_iter()
        .map(|l| {
            let mut best = -1;
            let mut best_x = 0;
            let mut best_y = 0;

            // Initialize the window for (1,1).
            let mut start_power = 0;
            for x in 1..=l {
                for y in 1..=l {
                    start_power += calc_power(x, y);
                }
            }

            // Now scan with a sliding window...
            for x in 1..=(300 - l) {
                // Slide the window along the y axis, checking each window for improvement.
                let mut power = start_power;
                for y in 1..=(300 - l) {
                    if power > best {
                        best = power;
                        best_x = x;
                        best_y = y;
                    }

                    // Moving down a row, so subtract the top row's power and add the next.
                    for offset in 0..l {
                        power -= calc_power(x + offset, y);
                        power += calc_power(x + offset, y + l);
                    }
                }

                // Now slide the start_power right by a column, and repeat.
                for y in 1..=l {
                    start_power -= calc_power(x, y);
                    start_power += calc_power(x + l, y);
                }
            }

            (best, best_x, best_y, l)
        })
        .max_by_key(|t| t.0)
        .unwrap();

    println!(
        "best power ({}) is {}x{} @ {},{}",
        best, best_l, best_l, best_x, best_y
    );

    Ok(())
}
