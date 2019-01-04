use failure::Error;

fn first_answer() {
    let mut board: Vec<usize> = vec![3, 7];
    let mut elf1 = 0;
    let mut elf2 = 1;

    let mut digits: Vec<usize> = Vec::new();

    let after = 768071;
    while board.len() < after + 10 {
        let score1 = board[elf1];
        let score2 = board[elf2];
        let mut score = score1 + score2;

        digits.push(score % 10);
        score /= 10;
        while score > 0 {
            digits.push(score % 10);
            score /= 10;
        }

        while !digits.is_empty() {
            board.push(digits.pop().unwrap());
        }

        let n = board.len();
        elf1 = (elf1 + 1 + score1) % n;
        elf2 = (elf2 + 1 + score2) % n;
    }

    println!("first answer: {:?}", &board[after..after + 10]);
}

const PATTERN: [usize; 6] = [7, 6, 8, 0, 7, 1];

struct Match {
    start: usize,
    next: usize,
}

impl Match {
    fn new(start: usize) -> Match {
        Match { start, next: 0 }
    }

    fn wants(&self, n: usize) -> bool {
        n == PATTERN[self.next]
    }
}

fn second_answer() {
    let mut board: Vec<usize> = vec![3, 7];
    let mut elf1 = 0;
    let mut elf2 = 1;

    let mut digits: Vec<usize> = Vec::new();

    let mut matches: Vec<Match> = Vec::new();

    loop {
        let score1 = board[elf1];
        let score2 = board[elf2];
        let mut score = score1 + score2;

        digits.push(score % 10);
        score /= 10;
        while score > 0 {
            digits.push(score % 10);
            score /= 10;
        }

        while !digits.is_empty() {
            matches.push(Match::new(board.len()));

            let recipe = digits.pop().unwrap();
            board.push(recipe);

            matches.retain(|m| m.wants(recipe));
            for m in matches.iter_mut() {
                m.next += 1;
                if m.next == PATTERN.len() {
                    println!("second answer: {}", m.start);
                    return;
                }
            }
        }

        let n = board.len();
        elf1 = (elf1 + 1 + score1) % n;
        elf2 = (elf2 + 1 + score2) % n;
    }
}

fn main() -> Result<(), Error> {
    first_answer();
    second_answer();

    Ok(())
}
