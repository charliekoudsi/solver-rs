#![feature(const_mut_refs)]
#![allow(warnings, unused)]
// mod best_response;
mod constants;
mod game;
mod regret;
mod terminal;
mod winners;
// use best_response::BestResponse;
use constants::{COMBOS, NO_DONK, NUM_CARDS, RIVER_CARDS, TURN_CARDS};
use crossbeam_utils::thread as crossbeam;
use game::{evaluate_winner, get_buckets, Game};
use ndarray::{arr1, array, Array1, Array2, ArrayBase, Dim, OwnedRepr};
use rand::{seq::SliceRandom, thread_rng};
use regret::{train, update_regret, RegretStrategy, SafeRegretStrategy};
use rs_poker::{gen_ranges, Card, Isomorph};
use std::{collections::HashMap, convert::TryInto, mem::size_of_val, time::Instant};
use terminal::get_index;
use winners::Winners;

// fn gen_range_cards(len: usize) -> Vec<usize> {
//     let mut r = vec![0; len];
//     for i in 0..len {
//         r[i] = i;
//     }
//     return r;
// }

// fn single_thread_train(
//     g: &Game,
//     strat: &mut [RegretStrategy; 2],
//     flop: &[u8; 3],
//     range1: &Vec<(u8, u8)>,
//     range2: &Vec<(u8, u8)>,
//     min_i: usize,
//     max_i: usize,
//     winners: &Winners,
// ) -> ([f32; 2], usize) {
//     let mut global_ev = [0.0; 2];
//     let mut combos = 0;
//     let mut rng = thread_rng();
//     let mut p2_range_cards = gen_range_cards(range2.len());
//     let mut turn_cards = TURN_CARDS.clone();
//     let mut river_cards = RIVER_CARDS.clone();
//     let mut board = [flop[0], flop[1], flop[2], 0, 0];
//     for i in min_i..max_i {
//         let (p1_one, p1_two) = range1[i];
//         p2_range_cards.shuffle(&mut rng);
//         for j in p2_range_cards.iter() {
//             let (p2_one, p2_two) = range2[*j];
//             if p1_one == p2_one || p1_one == p2_two || p1_two == p2_one || p1_two == p2_two {
//                 continue;
//             }
//             turn_cards.shuffle(&mut rng);
//             for t in turn_cards.iter() {
//                 let turn = *t;
//                 if turn == flop[0] || turn == flop[1] || turn == flop[2] {
//                     continue;
//                 }
//                 if turn == p1_one || turn == p1_two {
//                     continue;
//                 }
//                 if turn == p2_one || turn == p2_two {
//                     continue;
//                 }
//                 river_cards.shuffle(&mut rng);
//                 for r in river_cards.iter() {
//                     let river = *r;
//                     if river == flop[0]
//                         || river == flop[1]
//                         || river == flop[2]
//                         || river == turn
//                         || river == p1_one
//                         || river == p1_two
//                         || river == p2_one
//                         || river == p2_two
//                     {
//                         continue;
//                     }
//                     let ((p1_turn, p2_turn), (p1_river, p2_river)) =
//                         get_buckets(p1_one, p1_two, p2_one, p2_two, flop, turn, river);
//                     let buckets = [
//                         [i, i * 47 + p1_turn, (i * 47 + p1_turn) * 46 + p1_river],
//                         [*j, *j * 47 + p2_turn, (*j * 47 + p2_turn) * 46 + p2_river],
//                     ];
//                     board[3] = turn;
//                     board[4] = river;
//                     let result = winners.get_winner(i, *j, turn as usize, river as usize, 0);
//                     let mut ev = [0.0; 2];
//                     let mut reach = [1.0; 2];
//                     unsafe {
//                         update_regret(0, &buckets, result, &mut reach, 1.0, &mut ev, strat, g);
//                     }
//                     global_ev[0] += ev[0];
//                     global_ev[1] += ev[1];
//                     combos += 1;
//                 }
//             }
//         }
//     }
//     return (global_ev, combos);
// }

// fn train_4(
//     g: &Game,
//     strat_1: &mut [RegretStrategy; 2],
//     strat_2: &mut [RegretStrategy; 2],
//     strat_3: &mut [RegretStrategy; 2],
//     strat_4: &mut [RegretStrategy; 2],
//     flop: &[u8; 3],
//     range1: &Vec<(u8, u8)>,
//     range2: &Vec<(u8, u8)>,
//     winners: &Winners,
// ) -> (f32, [f32; 2]) {
//     let start = Instant::now();
//     // let combos = range1.len();
//     let v = crossbeam::scope(|scope| {
//         let a = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_1,
//                 flop,
//                 range1,
//                 range2,
//                 0,
//                 range1.len() / 4,
//                 winners,
//             );
//         });
//         let b = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_2,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() / 4,
//                 range1.len() / 2,
//                 winners,
//             );
//         });
//         let c = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_3,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() / 2,
//                 range1.len() * 3 / 4,
//                 winners,
//             );
//         });
//         let d = single_thread_train(
//             g,
//             strat_4,
//             flop,
//             range1,
//             range2,
//             range1.len() * 3 / 4,
//             range1.len(),
//             winners,
//         );
//         let a = a.join().unwrap();
//         let b = b.join().unwrap();
//         let c = c.join().unwrap();
//         let combos = a.1 + b.1 + c.1 + d.1;
//         let ev = [
//             (a.0[0] + b.0[0] + c.0[0] + d.0[0]) / combos as f32,
//             (a.0[1] + b.0[1] + c.0[1] + d.0[1]) / combos as f32,
//         ];
//         return ev;
//     })
//     .unwrap();
//     return (start.elapsed().as_secs_f32(), v);
// }

// fn train_8(
//     g: &Game,
//     strat_1: &mut [RegretStrategy; 2],
//     strat_2: &mut [RegretStrategy; 2],
//     strat_3: &mut [RegretStrategy; 2],
//     strat_4: &mut [RegretStrategy; 2],
//     strat_5: &mut [RegretStrategy; 2],
//     strat_6: &mut [RegretStrategy; 2],
//     strat_7: &mut [RegretStrategy; 2],
//     strat_8: &mut [RegretStrategy; 2],
//     flop: &[u8; 3],
//     range1: &Vec<(u8, u8)>,
//     range2: &Vec<(u8, u8)>,
//     winners: &Winners,
// ) -> (f32, [f32; 2]) {
//     let start = Instant::now();
//     // let combos = range1.len();
//     let v = crossbeam::scope(|scope| {
//         let a = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_1,
//                 flop,
//                 range1,
//                 range2,
//                 0,
//                 range1.len() / 8,
//                 winners,
//             );
//         });
//         let b = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_2,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() / 8,
//                 range1.len() / 4,
//                 winners,
//             );
//         });
//         let c = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_3,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() / 4,
//                 range1.len() * 3 / 8,
//                 winners,
//             );
//         });
//         let d = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_4,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() * 3 / 8,
//                 range1.len() / 2,
//                 winners,
//             );
//         });
//         let e = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_5,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() / 2,
//                 range1.len() * 5 / 8,
//                 winners,
//             );
//         });
//         let f = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_6,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() * 5 / 8,
//                 range1.len() * 3 / 4,
//                 winners,
//             );
//         });
//         let g_t = scope.spawn(move |_| {
//             return single_thread_train(
//                 g,
//                 strat_7,
//                 flop,
//                 range1,
//                 range2,
//                 range1.len() * 3 / 4,
//                 range1.len() * 7 / 8,
//                 winners,
//             );
//         });
//         let h = single_thread_train(
//             g,
//             strat_8,
//             flop,
//             range1,
//             range2,
//             range1.len() * 7 / 8,
//             range1.len(),
//             winners,
//         );
//         let a = a.join().unwrap();
//         let b = b.join().unwrap();
//         let c = c.join().unwrap();
//         let d = d.join().unwrap();
//         let e = e.join().unwrap();
//         let f = f.join().unwrap();
//         let g_t = g_t.join().unwrap();
//         let combos = a.1 + b.1 + c.1 + d.1 + e.1 + f.1 + g_t.1 + h.1;
//         let ev = [
//             (a.0[0] + b.0[0] + c.0[0] + d.0[0] + e.0[0] + f.0[0] + g_t.0[0] + h.0[0])
//                 / combos as f32,
//             (a.0[1] + b.0[1] + c.0[1] + d.0[1] + e.0[1] + f.0[1] + g_t.0[1] + h.0[1])
//                 / combos as f32,
//         ];
//         return ev;
//     })
//     .unwrap();
//     return (start.elapsed().as_secs_f32(), v);
// }

// struct Test {
//     ptr: *mut Vec<[ArrayBase<OwnedRepr<f32>, Dim<[usize; 1]>>; 3]>,
// }

// unsafe impl Send for Test {}

// fn t(ptr1: &mut Test, ptr2: &mut Test) {
//     crossbeam::scope(|scope| {
//         scope.spawn(move |_| unsafe {
//             (*(ptr1.ptr.offset(0)))[0][0][0] = 2.0;
//             // let t2 = &*ptr1.ptr.offset(1);
//             // let t1 = &*ptr1.ptr.offset(0);
//             // panic!("{}", t1 * t2);
//         });
//         scope.spawn(move |_| unsafe {
//             (*(ptr2.ptr.offset(0)))[0][0][1] = 3.0;
//             // let t2 = &*ptr2.ptr.offset(1);
//             // let t1 = &*ptr2.ptr.offset(0);
//             // panic!("{}", t1 * t2);
//         });
//     })
//     .unwrap();
// }

fn main() {
    let g = Game::new();
    let flop = [18, 31, 35];
    // let btn = [
    //     Isomorph::new_from_str("AA").unwrap(),
    //     Isomorph::new_from_str("AKs").unwrap(),
    //     Isomorph::new_from_str("AQs").unwrap(),
    //     Isomorph::new_from_str("AJs").unwrap(),
    //     Isomorph::new_from_str("ATs").unwrap(),
    //     Isomorph::new_from_str("A9s").unwrap(),
    //     Isomorph::new_from_str("A8s").unwrap(),
    //     Isomorph::new_from_str("A7s").unwrap(),
    //     Isomorph::new_from_str("A6s").unwrap(),
    //     Isomorph::new_from_str("A5s").unwrap(),
    //     Isomorph::new_from_str("A4s").unwrap(),
    //     Isomorph::new_from_str("A3s").unwrap(),
    //     Isomorph::new_from_str("A2s").unwrap(),
    //     Isomorph::new_from_str("AKo").unwrap(),
    //     Isomorph::new_from_str("AQo").unwrap(),
    //     Isomorph::new_from_str("AJo").unwrap(),
    //     Isomorph::new_from_str("ATo").unwrap(),
    //     Isomorph::new_from_str("A9o").unwrap(),
    //     Isomorph::new_from_str("A8o").unwrap(),
    //     Isomorph::new_from_str("A7o").unwrap(),
    //     Isomorph::new_from_str("A6o").unwrap(),
    //     Isomorph::new_from_str("A5o").unwrap(),
    //     Isomorph::new_from_str("A4o").unwrap(),
    //     Isomorph::new_from_str("KK").unwrap(),
    //     Isomorph::new_from_str("KQs").unwrap(),
    //     Isomorph::new_from_str("KJs").unwrap(),
    //     Isomorph::new_from_str("KTs").unwrap(),
    //     Isomorph::new_from_str("K9s").unwrap(),
    //     Isomorph::new_from_str("K8s").unwrap(),
    //     Isomorph::new_from_str("K7s").unwrap(),
    //     Isomorph::new_from_str("K6s").unwrap(),
    //     Isomorph::new_from_str("K5s").unwrap(),
    //     Isomorph::new_from_str("K4s").unwrap(),
    //     Isomorph::new_from_str("K3s").unwrap(),
    //     Isomorph::new_from_str("K2s").unwrap(),
    //     Isomorph::new_from_str("KQo").unwrap(),
    //     Isomorph::new_from_str("KJo").unwrap(),
    //     Isomorph::new_from_str("KTo").unwrap(),
    //     Isomorph::new_from_str("K9o").unwrap(),
    //     Isomorph::new_from_str("K8o").unwrap(),
    //     Isomorph::new_from_str("QQ").unwrap(),
    //     Isomorph::new_from_str("QJs").unwrap(),
    //     Isomorph::new_from_str("QTs").unwrap(),
    //     Isomorph::new_from_str("Q9s").unwrap(),
    //     Isomorph::new_from_str("Q8s").unwrap(),
    //     Isomorph::new_from_str("Q7s").unwrap(),
    //     Isomorph::new_from_str("Q6s").unwrap(),
    //     Isomorph::new_from_str("Q5s").unwrap(),
    //     Isomorph::new_from_str("Q4s").unwrap(),
    //     Isomorph::new_from_str("Q3s").unwrap(),
    //     Isomorph::new_from_str("Q2s").unwrap(),
    //     Isomorph::new_from_str("QJo").unwrap(),
    //     Isomorph::new_from_str("QTo").unwrap(),
    //     Isomorph::new_from_str("Q9o").unwrap(),
    //     Isomorph::new_from_str("JJ").unwrap(),
    //     Isomorph::new_from_str("JTs").unwrap(),
    //     Isomorph::new_from_str("J9s").unwrap(),
    //     Isomorph::new_from_str("J8s").unwrap(),
    //     Isomorph::new_from_str("J7s").unwrap(),
    //     Isomorph::new_from_str("J6s").unwrap(),
    //     Isomorph::new_from_str("J5s").unwrap(),
    //     Isomorph::new_from_str("J4s").unwrap(),
    //     Isomorph::new_from_str("JTo").unwrap(),
    //     Isomorph::new_from_str("J9o").unwrap(),
    //     Isomorph::new_from_str("J8o").unwrap(),
    //     Isomorph::new_from_str("TT").unwrap(),
    //     Isomorph::new_from_str("T9s").unwrap(),
    //     Isomorph::new_from_str("T8s").unwrap(),
    //     Isomorph::new_from_str("T7s").unwrap(),
    //     Isomorph::new_from_str("T6s").unwrap(),
    //     Isomorph::new_from_str("T9o").unwrap(),
    //     Isomorph::new_from_str("T8o").unwrap(),
    //     Isomorph::new_from_str("99").unwrap(),
    //     Isomorph::new_from_str("98s").unwrap(),
    //     Isomorph::new_from_str("97s").unwrap(),
    //     Isomorph::new_from_str("96s").unwrap(),
    //     Isomorph::new_from_str("98o").unwrap(),
    //     Isomorph::new_from_str("88").unwrap(),
    //     Isomorph::new_from_str("87s").unwrap(),
    //     Isomorph::new_from_str("86s").unwrap(),
    //     Isomorph::new_from_str("77").unwrap(),
    //     Isomorph::new_from_str("76s").unwrap(),
    //     Isomorph::new_from_str("75s").unwrap(),
    //     Isomorph::new_from_str("66").unwrap(),
    //     Isomorph::new_from_str("65s").unwrap(),
    //     Isomorph::new_from_str("55").unwrap(),
    //     Isomorph::new_from_str("54s").unwrap(),
    //     Isomorph::new_from_str("44").unwrap(),
    //     Isomorph::new_from_str("33").unwrap(),
    //     Isomorph::new_from_str("22").unwrap(),
    // ];
    // let bb = [
    //     Isomorph::new_from_str("AJs").unwrap(),
    //     Isomorph::new_from_str("ATs").unwrap(),
    //     Isomorph::new_from_str("A9s").unwrap(),
    //     Isomorph::new_from_str("A8s").unwrap(),
    //     Isomorph::new_from_str("A7s").unwrap(),
    //     Isomorph::new_from_str("A6s").unwrap(),
    //     Isomorph::new_from_str("A5s").unwrap(),
    //     Isomorph::new_from_str("A4s").unwrap(),
    //     Isomorph::new_from_str("A3s").unwrap(),
    //     Isomorph::new_from_str("A2s").unwrap(),
    //     Isomorph::new_from_str("AQo").unwrap(),
    //     Isomorph::new_from_str("AJo").unwrap(),
    //     Isomorph::new_from_str("ATo").unwrap(),
    //     Isomorph::new_from_str("A9o").unwrap(),
    //     Isomorph::new_from_str("A8o").unwrap(),
    //     Isomorph::new_from_str("A7o").unwrap(),
    //     Isomorph::new_from_str("A6o").unwrap(),
    //     Isomorph::new_from_str("A5o").unwrap(),
    //     Isomorph::new_from_str("A4o").unwrap(),
    //     Isomorph::new_from_str("KQs").unwrap(),
    //     Isomorph::new_from_str("KJs").unwrap(),
    //     Isomorph::new_from_str("KTs").unwrap(),
    //     Isomorph::new_from_str("K9s").unwrap(),
    //     Isomorph::new_from_str("K8s").unwrap(),
    //     Isomorph::new_from_str("K7s").unwrap(),
    //     Isomorph::new_from_str("K6s").unwrap(),
    //     Isomorph::new_from_str("K5s").unwrap(),
    //     Isomorph::new_from_str("K4s").unwrap(),
    //     Isomorph::new_from_str("K3s").unwrap(),
    //     Isomorph::new_from_str("K2s").unwrap(),
    //     Isomorph::new_from_str("KQo").unwrap(),
    //     Isomorph::new_from_str("KJo").unwrap(),
    //     Isomorph::new_from_str("KTo").unwrap(),
    //     Isomorph::new_from_str("K9o").unwrap(),
    //     Isomorph::new_from_str("K8o").unwrap(),
    //     Isomorph::new_from_str("QJs").unwrap(),
    //     Isomorph::new_from_str("QTs").unwrap(),
    //     Isomorph::new_from_str("Q9s").unwrap(),
    //     Isomorph::new_from_str("Q8s").unwrap(),
    //     Isomorph::new_from_str("Q7s").unwrap(),
    //     Isomorph::new_from_str("Q6s").unwrap(),
    //     Isomorph::new_from_str("Q5s").unwrap(),
    //     Isomorph::new_from_str("Q4s").unwrap(),
    //     Isomorph::new_from_str("Q3s").unwrap(),
    //     Isomorph::new_from_str("Q2s").unwrap(),
    //     Isomorph::new_from_str("QJo").unwrap(),
    //     Isomorph::new_from_str("QTo").unwrap(),
    //     Isomorph::new_from_str("Q9o").unwrap(),
    //     Isomorph::new_from_str("J8s").unwrap(),
    //     Isomorph::new_from_str("J7s").unwrap(),
    //     Isomorph::new_from_str("J6s").unwrap(),
    //     Isomorph::new_from_str("J5s").unwrap(),
    //     Isomorph::new_from_str("J4s").unwrap(),
    //     Isomorph::new_from_str("JTo").unwrap(),
    //     Isomorph::new_from_str("J9o").unwrap(),
    //     Isomorph::new_from_str("T8s").unwrap(),
    //     Isomorph::new_from_str("T7s").unwrap(),
    //     Isomorph::new_from_str("T6s").unwrap(),
    //     Isomorph::new_from_str("T9o").unwrap(),
    //     Isomorph::new_from_str("T8o").unwrap(),
    //     Isomorph::new_from_str("99").unwrap(),
    //     Isomorph::new_from_str("98s").unwrap(),
    //     Isomorph::new_from_str("97s").unwrap(),
    //     Isomorph::new_from_str("96s").unwrap(),
    //     Isomorph::new_from_str("98o").unwrap(),
    //     Isomorph::new_from_str("88").unwrap(),
    //     Isomorph::new_from_str("87s").unwrap(),
    //     Isomorph::new_from_str("86s").unwrap(),
    //     Isomorph::new_from_str("85s").unwrap(),
    //     Isomorph::new_from_str("87o").unwrap(),
    //     Isomorph::new_from_str("77").unwrap(),
    //     Isomorph::new_from_str("76s").unwrap(),
    //     Isomorph::new_from_str("75s").unwrap(),
    //     Isomorph::new_from_str("74s").unwrap(),
    //     Isomorph::new_from_str("76o").unwrap(),
    //     Isomorph::new_from_str("66").unwrap(),
    //     Isomorph::new_from_str("65s").unwrap(),
    //     Isomorph::new_from_str("64s").unwrap(),
    //     Isomorph::new_from_str("63s").unwrap(),
    //     Isomorph::new_from_str("65o").unwrap(),
    //     Isomorph::new_from_str("55").unwrap(),
    //     Isomorph::new_from_str("54s").unwrap(),
    //     Isomorph::new_from_str("53s").unwrap(),
    //     Isomorph::new_from_str("52s").unwrap(),
    //     Isomorph::new_from_str("44").unwrap(),
    //     Isomorph::new_from_str("43s").unwrap(),
    //     Isomorph::new_from_str("42s").unwrap(),
    //     Isomorph::new_from_str("33").unwrap(),
    //     Isomorph::new_from_str("22").unwrap(),
    // ];
    let combos = [
        Isomorph::new(12, 12, false),
        Isomorph::new(12, 11, true),
        Isomorph::new(11, 11, false),
        Isomorph::new(11, 10, true),
        Isomorph::new(12, 10, true),
        Isomorph::new(10, 10, false),
        Isomorph::new(10, 9, true),
        Isomorph::new(9, 9, false),
        Isomorph::new(9, 8, true),
        Isomorph::new(8, 8, false),
        Isomorph::new(7, 7, false),
        Isomorph::new(8, 7, true),
        Isomorph::new(6, 5, true),
    ];
    let range1 = gen_ranges(&combos, &flop);
    let mut chance1 = Array1::<f32>::zeros(COMBOS);
    let len = range1.len() as f32;
    for (c1, c2) in range1 {
        chance1[get_index(c1, c2)] = 1.0;
    }
    println!("{}", len);
    let mut chance2 = chance1.clone();
    let chance = [chance1, chance2];
    // let range2 = gen_ranges(&combos, &flop);

    // println!("generating map");
    // let time = Instant::now();
    // let map = Winners::new(&range1, &range2, &flop);
    // println!("elapsed: {}", time.elapsed().as_secs_f32());
    let mut safe_1 = SafeRegretStrategy::new(&g, 0);
    let mut safe_2 = SafeRegretStrategy::new(&g, 1);
    // let regret_1 = RegretStrategy::new_4(&mut safe_1.regret, &mut safe_1.average_probability);
    // let regret_2 = RegretStrategy::new_4(&mut safe_2.regret, &mut safe_2.average_probability);
    let mut strat = RegretStrategy::new(
        &mut safe_1.regret,
        &mut safe_1.average_probability,
        &mut safe_2.regret,
        &mut safe_2.average_probability,
        1,
    );
    for _ in 0..1 {
        println!("starting training");
        let time = Instant::now();
        train(&flop, &chance, &mut strat[..], &g);
        println!("elapsed, {}", time.elapsed().as_secs_f32());
    }
    // unsafe {
    //     println!(
    //         "{:?}",
    //         strat[0][1].get_regret(g.transition[0][1].try_into().unwrap(), 0)
    //     );
    // }

    let avg =
        safe_2.get_average_normalized_probability(g.transition[0][1].try_into().unwrap(), 0, &g);
    let mut idx = 0;
    for i in 0..52 {
        for j in (i + 1)..52 {
            if chance[1][idx] != 0.0 {
                let prob = [avg[[0, idx]], avg[[1, idx]], avg[[2, idx]]];
                let c1 = Card::from_u8(i);
                let c2 = Card::from_u8(j);
                println!(
                    "{}{}{}{}: {:?}",
                    c1.value.to_char(),
                    c1.suit.to_char(),
                    c2.value.to_char(),
                    c2.suit.to_char(),
                    prob
                );
            }
            idx += 1;
        }
    }

    // // let mut strat_2 = [regret_1.1, regret_2.1];
    // // let mut strat_3 = [regret_1.2, regret_2.2];
    // // let mut strat_4 = [regret_1.3, regret_2.3];
    // let mut dEV = 100.0;
    // let mut runs = 0;
    // // while dEV > 0.5 {
    // let mut ev = 0.0;
    // // {
    // let regret_1 = RegretStrategy::new_8(
    //     &mut safe_1.regret,
    //     &mut safe_1.average_probability,
    //     &mut safe_1.updates,
    // );
    // let regret_2 = RegretStrategy::new_8(
    //     &mut safe_2.regret,
    //     &mut safe_2.average_probability,
    //     &mut safe_2.updates,
    // );
    // let mut strat_1 = [regret_1.0, regret_2.0];
    // let mut strat_2 = [regret_1.1, regret_2.1];
    // let mut strat_3 = [regret_1.2, regret_2.2];
    // let mut strat_4 = [regret_1.3, regret_2.3];
    // let mut strat_5 = [regret_1.4, regret_2.4];
    // let mut strat_6 = [regret_1.5, regret_2.5];
    // let mut strat_7 = [regret_1.6, regret_2.6];
    // let mut strat_8 = [regret_1.7, regret_2.7];
    // println!("starting training");
    // let mut total = 0.0;
    // while total < 2000.0 {
    //     let (time, run_ev) = train_8(
    //         &g,
    //         &mut strat_1,
    //         &mut strat_2,
    //         &mut strat_3,
    //         &mut strat_4,
    //         &mut strat_5,
    //         &mut strat_6,
    //         &mut strat_7,
    //         &mut strat_8,
    //         &flop,
    //         &range1,
    //         &range2,
    //         &map,
    //     );
    //     total += time;
    //     println!("{:?}", run_ev);
    //     println!("{}: {}", runs, time);
    //     ev = run_ev[0] + run_ev[1];
    //     runs += 1;
    // }

    // let mut best_resp_strat = SafeRegretStrategy::new(&g, 0, range1.len());
    // let best_resp = BestResponse::new(0, &safe_2, &g, range1.clone(), range2.clone(), flop.clone());
    // println!("computing br");
    // let time = Instant::now();
    // let mut val =
    //     best_resp.compute_best_response(0, &g, &mut best_resp_strat, None, None, None, &map);

    // println!("{} ({})", val, time.elapsed().as_secs_f32());
    // let mut best_resp_strat = SafeRegretStrategy::new(&g, 1, range2.len());
    // let best_resp = BestResponse::new(1, &safe_1, &g, range2.clone(), range1.clone(), flop.clone());
    // let new_val =
    //     best_resp.compute_best_response(0, &g, &mut best_resp_strat, None, None, None, &map);
    // println!("[{}, {}]", val, new_val);
    // val += new_val;
    // println!("{} ({})", val, time.elapsed().as_secs_f32());
    // dEV = 100.0 * (val - ev) / val;
    // let exploitability = (val - ev) / 4.0;
    // println!(
    //     "dEV: {}, Exploitability:  {}, Elapsed: {}",
    //     dEV,
    //     exploitability,
    //     time.elapsed().as_secs_f32()
    // );
    // let mut i = 0;
    // for (c1, c2) in &range2 {
    //     let card1 = Card::from_u8(*c1);
    //     let card2 = Card::from_u8(*c2);
    //     println!(
    //         "P2 vs. Check {}{}{}{}: {:?}",
    //         card1.value.to_char(),
    //         card1.suit.to_char(),
    //         card2.value.to_char(),
    //         card2.suit.to_char(),
    //         safe_2.get_average_normalized_probability(
    //             g.transition[0][1].try_into().unwrap(),
    //             i,
    //             &g
    //         )
    //     );
    //     if !NO_DONK {
    //         println!(
    //             "P2 vs. Bet {}{}{}{}: {:?}",
    //             card1.value.to_char(),
    //             card1.suit.to_char(),
    //             card2.value.to_char(),
    //             card2.suit.to_char(),
    //             safe_2.get_average_normalized_probability(
    //                 g.transition[0][0].try_into().unwrap(),
    //                 i,
    //                 &g
    //             )
    //         );
    //     }
    //     i += 1;
    //     // let encoded: Vec<u8> = bincode::serialize(&strat.lock().unwrap()[0]).unwrap();
    //     // let mut file = File::create("test").unwrap();
    //     // file.write_all(&encoded).unwrap();
    // }
    // println!("{}", total / runs as f32);
    // println!("{}", safe_1.updates[0][55]);
}
