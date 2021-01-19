use std::cmp::min;
mod game;
mod regret;
use game::{evaluate_winner, Game};
use hand_eval::Card;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
// const CARDS: [[i32; 36]; 35] = gen_cards();
// fn count_sequences(
//     round: usize,
//     raises: usize,
//     first_action: bool,
//     internal: &mut usize,
//     terminal: &mut usize,
// ) {
//     *internal += 1;

//     // can we raise?
//     if raises < 2 {
//         count_sequences(round, raises + 1, false, internal, terminal);
//     }

//     // can we check/call?
//     if first_action {
//         // we can always check if first up
//         count_sequences(round, raises, false, internal, terminal);
//     } else {
//         if round != 0 {
//             // if it's last round, check/call ends everything
//             *terminal += 1;
//         } else {
//             count_sequences(1, 0, true, internal, terminal);
//         }
//     }

//     // can we fold?
//     if raises != 0 {
//         *terminal += 1;
//     }
// }

fn count_sequences(
    player: usize,
    round: usize,
    raise: i32,
    first_action: bool,
    pot: i32,
    stack: i32,
    internal: &mut usize,
    terminal: &mut usize,
) {
    let opponent = 1 - player;
    *internal += 1;
    if stack > 0 {
        // if player == 0 && round != 0 {
        //     if stack > pot + raise {
        //         count_sequences(
        //             opponent,
        //             round,
        //             pot + raise,
        //             false,
        //             pot + raise + pot + raise,
        //             stack - (pot + raise),
        //             internal,
        //             terminal,
        //         );
        //         count_sequences(
        //             opponent,
        //             round,
        //             (pot + raise) * 3 / 4,
        //             false,
        //             pot + raise + (pot + raise) * 3 / 4,
        //             stack - (pot + raise) * 3 / 4,
        //             internal,
        //             terminal,
        //         );
        //         count_sequences(
        //             opponent,
        //             round,
        //             (pot + raise) / 2,
        //             false,
        //             pot + raise + (pot + raise) / 2,
        //             stack - (pot + raise) / 2,
        //             internal,
        //             terminal,
        //         );
        //     } else if stack > (pot + raise) * 3 / 4 {
        //         count_sequences(
        //             opponent,
        //             round,
        //             (pot + raise) * 3 / 4,
        //             false,
        //             pot + raise + (pot + raise) * 3 / 4,
        //             stack - (pot + raise) * 3 / 4,
        //             internal,
        //             terminal,
        //         );
        //         count_sequences(
        //             opponent,
        //             round,
        //             (pot + raise) / 2,
        //             false,
        //             pot + raise + (pot + raise) / 2,
        //             stack - (pot + raise) / 2,
        //             internal,
        //             terminal,
        //         );
        //     } else if stack > (pot + raise) / 2 {
        //         count_sequences(
        //             opponent,
        //             round,
        //             (pot + raise) / 2,
        //             false,
        //             pot + raise + (pot + raise) / 2,
        //             stack - (pot + raise) / 2,
        //             internal,
        //             terminal,
        //         );
        // } else {
        // count_sequences(
        //     opponent,
        //     round,
        //     stack,
        //     false,
        //     pot + raise + stack,
        //     0,
        //     internal,
        //     terminal,
        // );
        // }
        // } else if player == 1 {
        let raise_size = min((pot + raise) / 2, stack);
        count_sequences(
            opponent,
            round,
            raise_size,
            false,
            pot + raise + raise_size,
            stack - raise_size,
            internal,
            terminal,
        );
        // }
    }

    if first_action {
        // if stack > 0 {
        count_sequences(
            opponent, round, raise, false, pot, stack, internal, terminal,
        );
    // }
    } else {
        if round == 2 {
            *terminal += 1;
        } else {
            if stack > 0 || raise != 0 {
                count_sequences(
                    0,
                    round + 1,
                    0,
                    true,
                    pot + raise,
                    stack,
                    internal,
                    terminal,
                );
            } else {
                *terminal += 1;
            }
        }
    }

    if raise != 0 {
        *terminal += 1;
    }

    // println!(
    //     "Player: {}, Round: {}, Raise: {}, First: {}, Pot: {}, Stack: {}",
    //     player, round, raise, first_action, pot, stack
    // );
}

//

use regret::{update_regret, RegretStrategy};
fn train(
    g: &Game,
    strat: &mut [RegretStrategy; 2],
    // strat: Arc<Mutex<[RegretStrategy; 2]>>,
    cum_ev: &mut [f64; 2],
    flop: &[u8; 3],
    range1: &Vec<(u8, u8)>,
    range2: &Vec<(u8, u8)>,
    iteration: &mut u64,
    acfr: &mut [f64; 2],
) -> f64 {
    let start = Instant::now();
    // let combos = range1.len();
    let mut i = 0;
    // let mut runs = 0;
    for (p1_one, p1_two) in range1 {
        let mut j = 0;
        for (p2_one, p2_two) in range2 {
            if p1_one == p2_one || p1_one == p2_two || p1_two == p2_one || p1_two == p2_two {
                j += 1;
                continue;
            }
            let mut p1_turn = 0;
            let mut p2_turn = 0;
            for turn in 0..36 {
                if turn == flop[0] || turn == flop[1] || turn == flop[2] {
                    continue;
                }
                if turn == *p1_one || turn == *p1_two {
                    p2_turn += 1;
                    continue;
                }
                if turn == *p2_one || turn == *p2_two {
                    p1_turn += 1;
                    continue;
                }
                let mut p1_river = 0;
                let mut p2_river = 0;
                // (0..36u8).into_par_iter().for_each(|river| {
                //     // let strategy = Arc::clone(&strat);
                //     let mut halt = false;
                //     if river as u8 == flop[0]
                //         || river == flop[1]
                //         || river == flop[2]
                //         || river == turn
                //     {
                //         halt = true;
                //     } else if river == *p1_one || river == *p1_two {
                //         // p2_river += 1;
                //         halt = true;
                //     } else if river == *p2_one || river == *p2_two {
                //         // p1_river += 1;
                //         halt = true;
                //     }
                //     if !halt {
                //         let buckets = [
                //             [i, i * 31 + p1_turn, (i * 31 + p1_turn) * 30 + p1_river],
                //             [j, j * 31 + p2_turn, (j * 31 + p2_turn) * 30 + p2_river],
                //         ];
                //         let board = [flop[0], flop[1], flop[2], turn, river];
                //         let result =
                //             evaluate_winner((*p1_one, *p1_two), (*p2_one, *p2_two), &board);
                //         let mut ev = [1.0; 2];
                //         let mut reach = [1.0; 2];
                //         let mut cfr = [0.0; 2];
                //         // let mut guard = strat.lock().unwrap();
                //         // let protected = &mut *guard;
                //         update_regret(
                //             0, &buckets, result, &mut reach, 1.0, &mut ev, &mut cfr, strategy, g,
                //         );
                //     }
                //     // cum_ev[0] += ev[0];
                //     // cum_ev[1] += ev[1];
                //     // if *iteration == 1 {
                //     //     acfr[0] = cfr[0];
                //     //     acfr[1] = cfr[1];
                //     // } else {
                //     //     for c in 0..2 {
                //     //         acfr[c] = 1. * (*iteration as f64 - 1.) / *iteration as f64
                //     //             * (acfr[c] + cfr[c] / (*iteration as f64 - 1.));
                //     //     }
                //     // }
                //     // p1_river += 1;
                //     // p2_river += 1;
                //     // *iteration += 1;
                // });
                for river in 0..36 {
                    if river == flop[0] || river == flop[1] || river == flop[2] || river == turn {
                        continue;
                    }
                    if river == *p1_one || river == *p1_two {
                        p2_river += 1;
                        continue;
                    }
                    if river == *p2_one || river == *p2_two {
                        p1_river += 1;
                        continue;
                    }
                    // if turn == 1 && river == 35 && *p2_one == 7 && *p2_two == 11 {
                    //     println!("{}", (j * 31 + p2_turn));
                    // }
                    // if (j * 31 + p2_turn) == 1706 {
                    //     assert_eq!(turn, 1, "turn");
                    //     assert_eq!(*p2_one, 7, "c1");
                    //     assert_eq!(*p2_two, 11, "c2");
                    // }
                    let buckets = [
                        [i, i * 31 + p1_turn, (i * 31 + p1_turn) * 30 + p1_river],
                        [j, j * 31 + p2_turn, (j * 31 + p2_turn) * 30 + p2_river],
                    ];
                    // if i + combos * p1_turn == 1735 {
                    //     panic!("{} {} {}", i, combos, p1_turn);
                    // }
                    // if j + combos * p2_turn == 1735 {
                    //     panic!("{} {} {}", j, combos, p1_turn);
                    // }
                    let board = [flop[0], flop[1], flop[2], turn, river];
                    let result = evaluate_winner((*p1_one, *p1_two), (*p2_one, *p2_two), &board);
                    let mut ev = [1.0; 2];
                    let mut reach = [1.0; 2];
                    let mut cfr = [0.0; 2];
                    update_regret(
                        0, &buckets, result, &mut reach, 1.0, &mut ev, &mut cfr, strat, g,
                    );
                    // runs += 1;
                    cum_ev[0] += ev[0];
                    cum_ev[1] += ev[1];
                    if *iteration == 1 {
                        acfr[0] = cfr[0];
                        acfr[1] = cfr[1];
                    } else {
                        for c in 0..2 {
                            acfr[c] = 1. * (*iteration as f64 - 1.) / *iteration as f64
                                * (acfr[c] + cfr[c] / (*iteration as f64 - 1.));
                        }
                    }
                    p1_river += 1;
                    p2_river += 1;
                    *iteration += 1;
                }
                p1_turn += 1;
                p2_turn += 1;
            }
            j += 1;
        }
        i += 1;
    }
    println!("{}", *iteration);
    return start.elapsed().as_secs_f64();
}

use hand_eval::{gen_range, Isomorph};
use std::fs::File;
use std::io::prelude::*;
fn main() {
    let g = Game::new();
    let flop = [19, 15, 2];
    let combos = [
        Isomorph::new(8, 8, false),
        Isomorph::new(8, 7, true),
        Isomorph::new(7, 7, false),
        Isomorph::new(7, 6, true),
        Isomorph::new(8, 6, true),
        Isomorph::new(6, 6, false),
        Isomorph::new(6, 5, true),
        Isomorph::new(5, 5, false),
        Isomorph::new(5, 4, true),
        Isomorph::new(4, 4, false),
        Isomorph::new(3, 3, false),
        Isomorph::new(4, 3, true),
        Isomorph::new(2, 1, true),
    ];
    let range1 = gen_range(&combos, &flop);
    let range2 = gen_range(&combos, &flop);
    println!("{:?}", range1);
    let mut strat = [
        RegretStrategy::new(0, &g, range1.len()),
        RegretStrategy::new(1, &g, range1.len()),
    ];
    let mut cum_ev = [0.0; 2];
    let mut acfr = [0.0; 2];
    let mut iteration = 0;
    let mut total = 0.0;
    for _ in 0..25 {
        // let strategy = Arc::clone(&strat);
        total += train(
            &g,
            &mut strat,
            &mut cum_ev,
            &flop,
            &range1,
            &range2,
            &mut iteration,
            &mut acfr,
        );
        println!("{:?}", acfr);
    }
    // println!(
    //     "{:?}",
    //     strat[1].get_average_normalized_probability(1, 33, &g)
    // );
    // println!(
    //     "{:?}",
    //     strat[1].get_average_normalized_probability(83, 33, &g)
    // );
    // println!(
    //     "{:?}",
    //     strat[0].get_average_normalized_probability(0, 1, &g)
    // );
    let mut i = 0;
    for (c1, c2) in range1 {
        let card1 = Card::from_u8(c1);
        let card2 = Card::from_u8(c2);
        println!(
            "P1 Open {}{}{}{}: {:?}",
            card1.value.to_char(),
            card1.suit.to_char(),
            card2.value.to_char(),
            card2.suit.to_char(),
            strat[0].get_average_normalized_probability(0, i, &g)
        );
        println!(
            "P2 vs. Check {}{}{}{}: {:?}",
            card1.value.to_char(),
            card1.suit.to_char(),
            card2.value.to_char(),
            card2.suit.to_char(),
            strat[1].get_average_normalized_probability(83, i, &g)
        );
        println!(
            "P2 vs. Bet {}{}{}{}: {:?}",
            card1.value.to_char(),
            card1.suit.to_char(),
            card2.value.to_char(),
            card2.suit.to_char(),
            strat[1].get_average_normalized_probability(1, i, &g)
        );
        i += 1;
        // let encoded: Vec<u8> = bincode::serialize(&strat.lock().unwrap()[0]).unwrap();
        // let mut file = File::create("test").unwrap();
        // file.write_all(&encoded).unwrap();
    }
    println!("{}", total / 25.0);
}
