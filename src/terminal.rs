use crate::constants::{Array1, COMBOS, HOLES, STARTING_POT};
use crate::game::get_rank;
use nalgebra::{SVector, Scalar};
use rs_poker::Rank;

// Maybe: use SoA instead where each Array can be an Array1
#[derive(Debug, PartialEq, Clone)]
pub struct RankedHand {
    c1: u8,
    c2: u8,
    rank: Rank,
    index: usize,
}

impl RankedHand {
    fn new(c1: u8, c2: u8, rank: Rank, index: usize) -> Self {
        Self {
            c1,
            c2,
            rank,
            index,
        }
    }
    fn default(index: usize) -> Self {
        Self {
            c1: 0,
            c2: 0,
            rank: Rank::HighCard(0),
            index,
        }
    }
}

impl Scalar for RankedHand {}

pub type RankedArray = SVector<RankedHand, COMBOS>;

// I'm not sure I actually need all possible combos
pub fn rank_board(board: &[u8; 5]) -> RankedArray {
    let mut ranks = Vec::with_capacity(COMBOS);
    let mut idx = 0;
    for i in 0..52 {
        if i != board[0] && i != board[1] && i != board[2] && i != board[3] && i != board[4] {
            for j in (i + 1)..52 {
                if j != board[0] && j != board[1] && j != board[2] && j != board[3] && j != board[4]
                {
                    ranks.push(RankedHand::new(i, j, get_rank(i, j, board), idx));
                } else {
                    // TODO: add unrankable variant to Rank
                    ranks.push(RankedHand::default(idx));
                }
                idx += 1;
            }
        } else {
            // TODO: add unrankable variant to Rank
            for j in (i + 1)..52 {
                ranks.push(RankedHand::default(idx));
                idx += 1;
            }
        }
    }
    ranks.sort_unstable_by(|r1, r2| r1.rank.cmp(&r2.rank));
    return RankedArray::from_vec(ranks);
}

pub fn get_index(card1: u8, card2: u8) -> usize {
    (0..52).rev().take(card1 as usize).fold(0, |a, b| a + b) + (card2 - card1) as usize - 1

    // Test:
    // let mut idx = 0;
    // for i in 0..52 {
    //     for j in (i + 1)..52 {
    //         assert_eq!(idx, terminal::get_index(i, j), "{}, {}", i, j);
    //         idx += 1;
    //     }
    // }
}

pub fn eval_showdown(sd_value: f32, hands: &RankedArray, opp_probs: &Array1) -> Array1 {
    let num_hands = COMBOS;
    let mut sum: f32 = 0.0;
    let mut i = 0;
    let mut sum_including_card: [f32; 52] = [0.0; 52];
    let mut result: Array1 = Array1::zeros();
    for k in 0..num_hands {
        if opp_probs[hands[k].index] > 0.0 {
            sum_including_card[hands[k].c1 as usize] -= opp_probs[hands[k].index];
            sum_including_card[hands[k].c2 as usize] -= opp_probs[hands[k].index];
            sum -= opp_probs[hands[k].index];
        }
    }

    while i < num_hands {
        let mut j = i + 1;
        while j < num_hands && hands[j].rank == hands[i].rank {
            j += 1;
        }

        for k in i..j {
            sum_including_card[hands[k].c1 as usize] += opp_probs[hands[k].index];
            sum_including_card[hands[k].c2 as usize] += opp_probs[hands[k].index];
            sum += opp_probs[hands[k].index];
        }

        for k in i..j {
            // I'm not 100% sure this logic is right
            // TODO: Double check this
            let winner = sum
                - sum_including_card[hands[k].c1 as usize]
                - sum_including_card[hands[k].c2 as usize];
            if winner >= 0. {
                result[hands[k].index] = sd_value * winner;
            } else {
                // result[k] = (sd_value - (2 * STARTING_POT) as f32) * winner;
                result[hands[k].index] = sd_value * winner;
            }
        }

        for k in i..j {
            sum_including_card[hands[k].c1 as usize] += opp_probs[hands[k].index];
            sum_including_card[hands[k].c2 as usize] += opp_probs[hands[k].index];
            sum += opp_probs[hands[k].index];
        }
        i = j;
    }
    return result;
}

pub fn eval_fold(sd_value: f32, opp_probs: &Array1) -> Array1 {
    let mut result: Array1 = Array1::zeros();
    let mut sum: f32 = 0.0;
    let mut sum_including_card: [f32; 52] = [0.0; 52];

    for j in 0..COMBOS {
        sum_including_card[HOLES[j].0 as usize] += opp_probs[j];
        sum_including_card[HOLES[j].1 as usize] += opp_probs[j];
        sum += opp_probs[j];
    }

    for i in 0..COMBOS {
        result[i] = (sum
            - sum_including_card[HOLES[i].0 as usize]
            - sum_including_card[HOLES[i].1 as usize]
            + opp_probs[i])
            * sd_value;
        // if sd_value > 0.0 {
        //     assert!(
        //         result[i] >= 0.0,
        //         "not greater, {}, {}, {}, {}, {}, {}",
        //         result[i],
        //         sum,
        //         sum_including_card[HOLES[i].0 as usize],
        //         sum_including_card[HOLES[i].1 as usize],
        //         opp_probs[i],
        //         sd_value
        //     );
        // } else {
        //     assert!(result[i] <= 0.0, "not lesser");
        // }
    }

    return result;
}
