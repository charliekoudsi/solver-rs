use crate::constants::NUM_CARDS;
use crate::game::evaluate_winner;
use crossbeam_utils::thread as crossbeam;

type Range = Vec<(u8, u8)>;

pub struct Winners {
    pub map: Vec<Vec<[[i8; 52]; 52]>>,
}

impl Winners {
    pub fn new(range1: &Range, range2: &Range, flop: &[u8; 3]) -> Self {
        Self {
            map: Winners::multi_thread_gen_map(range1, range2, flop, 8),
        }
    }

    pub fn get_winner(
        &self,
        p1_index: usize,
        p2_index: usize,
        turn: usize,
        river: usize,
        player: usize,
    ) -> i8 {
        if player == 0 {
            return self.map[p1_index][p2_index][turn][river];
        } else {
            return -1 * self.map[p2_index][p1_index][turn][river];
        }
    }

    // DON'T DELETE THIS: USE TO TEST IMPLEMENTATION
    fn gen_map(range1: &Range, range2: &Range, flop: &[u8; 3]) -> Vec<Vec<[[i8; 52]; 52]>> {
        let mut map = vec![vec![[[0; 52]; 52]; range2.len()]; range1.len()];
        let mut board = [flop[0], flop[1], flop[2], 0, 0];
        let len = range1.len();
        for (i, hand1) in range1.iter().enumerate() {
            println!("{}/{}", i, len);
            for (j, hand2) in range2.iter().enumerate() {
                if hand1.0 == hand2.0
                    || hand1.0 == hand2.1
                    || hand1.1 == hand2.0
                    || hand1.1 == hand2.1
                {
                    continue;
                }
                for turn in 0..NUM_CARDS {
                    if hand1.0 == turn
                        || hand1.1 == turn
                        || hand2.0 == turn
                        || hand2.1 == turn
                        || flop[0] == turn
                        || flop[1] == turn
                        || flop[2] == turn
                    {
                        continue;
                    }
                    for river in 0..NUM_CARDS {
                        if turn == river
                            || hand1.0 == river
                            || hand1.1 == river
                            || hand2.0 == river
                            || hand2.1 == river
                            || flop[0] == river
                            || flop[1] == river
                            || flop[2] == river
                        {
                            continue;
                        }
                        board[3] = turn;
                        board[4] = river;
                        let winner = evaluate_winner(*hand1, *hand2, &board);
                        map[i][j][turn as usize][river as usize] = winner;
                    }
                }
            }
        }

        return map;
    }

    fn multi_thread_gen_map(
        range1: &Range,
        range2: &Range,
        flop: &[u8; 3],
        num_threads: usize,
    ) -> Vec<Vec<[[i8; 52]; 52]>> {
        let mut map = vec![vec![[[0; 52]; 52]; range2.len()]; range1.len()];
        let mut end = 0;
        let chunk_size = range1.len() / num_threads;
        crossbeam::scope(|scope| {
            for slice in map.chunks_mut(chunk_size) {
                end += slice.len();
                scope.spawn(move |_| {
                    Winners::single_thread_mut_map(
                        range1,
                        range2,
                        flop,
                        slice,
                        end - slice.len(),
                        end,
                    );
                });
            }
        })
        .unwrap();
        return map;
    }

    fn single_thread_mut_map(
        range1: &Range,
        range2: &Range,
        flop: &[u8; 3],
        winners: &mut [Vec<[[i8; 52]; 52]>],
        start: usize,
        end: usize,
    ) {
        let mut board = [flop[0], flop[1], flop[2], 0, 0];
        for (i, index) in (start..end).into_iter().enumerate() {
            let hand1 = range1[index];
            for (j, hand2) in range2.iter().enumerate() {
                if hand1.0 == hand2.0
                    || hand1.0 == hand2.1
                    || hand1.1 == hand2.0
                    || hand1.1 == hand2.1
                {
                    continue;
                }
                for turn in 0..NUM_CARDS {
                    if hand1.0 == turn
                        || hand1.1 == turn
                        || hand2.0 == turn
                        || hand2.1 == turn
                        || flop[0] == turn
                        || flop[1] == turn
                        || flop[2] == turn
                    {
                        continue;
                    }
                    for river in 0..NUM_CARDS {
                        if turn == river
                            || hand1.0 == river
                            || hand1.1 == river
                            || hand2.0 == river
                            || hand2.1 == river
                            || flop[0] == river
                            || flop[1] == river
                            || flop[2] == river
                        {
                            continue;
                        }
                        board[3] = turn;
                        board[4] = river;
                        let winner = evaluate_winner(hand1, *hand2, &board);
                        winners[i][j][turn as usize][river as usize] = winner;
                    }
                }
            }
        }
    }
}
