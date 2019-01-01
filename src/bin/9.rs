use std::collections::VecDeque;

use failure::Error;

fn rotate_cw(circle: &mut VecDeque<usize>, n: usize) {
    for _ in 0..n {
        let marble = circle.pop_front().unwrap();
        circle.push_back(marble);
    }
}

fn rotate_ccw(circle: &mut VecDeque<usize>, n: usize) {
    for _ in 0..n {
        let marble = circle.pop_back().unwrap();
        circle.push_front(marble);
    }
}

/// Run the game with @p players until marble @p last_marble.
fn game_vec(players: usize, last_marble: usize) {
    // 100x larger eh?  That's cute.
    // We'll use a VecDeque and keep the "current" marble at index 0.
    let mut circle: VecDeque<usize> = vec![0].into_iter().collect();
    let mut next_marble = 1;
    let mut scores = vec![0; players];
    let mut player = 0;

    while next_marble <= last_marble {
        match next_marble % 23 {
            0 => {
                rotate_ccw(&mut circle, 6);
                let removed = circle.pop_back().unwrap();
                scores[player] += next_marble + removed;
            }
            _ => {
                rotate_cw(&mut circle, 2);
                circle.push_front(next_marble);
            }
        }
        next_marble += 1;
        player = (player + 1) % players;
    }

    let (winner, high_score) = scores
        .iter()
        .enumerate()
        .max_by_key(|(_, &score)| score)
        .unwrap();

    println!(
        "{} players; last marble is worth {} points: high score is {} [player {}]",
        players,
        last_marble,
        high_score,
        winner + 1
    );
}

fn main() -> Result<(), Error> {
    game_vec(10, 1618);
    game_vec(13, 7999);
    game_vec(17, 1104);
    game_vec(21, 6111);
    game_vec(30, 5807);
    game_vec(468, 71843);
    game_vec(468, 71843 * 100);
    Ok(())
}
