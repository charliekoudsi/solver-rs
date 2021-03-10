#![feature(const_mut_refs)]
mod best_response;
mod game;
mod regret;
use best_response::BestResponse;
use game::{evaluate_winner, get_buckets, Game};
use hand_eval::Card;
use hand_eval::{gen_ranges, Isomorph};
use rand::{seq::SliceRandom, thread_rng};
use regret::{update_regret, RegretStrategy, SafeRegretStrategy};
use std::{convert::TryInto, time::Instant};

fn single_thread_train(
    g: &Game,
    strat: &mut [RegretStrategy; 2],
    flop: &[u8; 3],
    range1: &Vec<(u8, u8)>,
    range2: &Vec<(u8, u8)>,
    min_i: usize,
    max_i: usize,
) -> ([f64; 2], usize) {
    let mut global_ev = [0.0; 2];
    let mut combos = 0;
    let mut rng = thread_rng();
    let mut turn_cards = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    ];
    let mut river_cards = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    ];
    let mut p2_range_cards = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55,
    ];
    for i in min_i..max_i {
        let (p1_one, p1_two) = range1[i];
        p2_range_cards.shuffle(&mut rng);
        for j in p2_range_cards.iter() {
            let (p2_one, p2_two) = range2[*j];
            if p1_one == p2_one || p1_one == p2_two || p1_two == p2_one || p1_two == p2_two {
                continue;
            }
            turn_cards.shuffle(&mut rng);
            for t in 0..36 {
                let turn = turn_cards[t];
                if turn == flop[0] || turn == flop[1] || turn == flop[2] {
                    continue;
                }
                if turn == p1_one || turn == p1_two {
                    continue;
                }
                if turn == p2_one || turn == p2_two {
                    continue;
                }
                river_cards.shuffle(&mut rng);
                for r in 0..36 {
                    let river = river_cards[r];
                    if river == flop[0]
                        || river == flop[1]
                        || river == flop[2]
                        || river == turn
                        || river == p1_one
                        || river == p1_two
                        || river == p2_one
                        || river == p2_two
                    {
                        continue;
                    }
                    let ((p1_turn, p2_turn), (p1_river, p2_river)) =
                        get_buckets(p1_one, p1_two, p2_one, p2_two, flop, turn, river);
                    let buckets = [
                        [i, i * 31 + p1_turn, (i * 31 + p1_turn) * 30 + p1_river],
                        [*j, *j * 31 + p2_turn, (*j * 31 + p2_turn) * 30 + p2_river],
                    ];
                    // if (i * 31 + p1_turn) * 30 + p1_river >= 52080 {
                    //     panic!(
                    //         "{} {} {} {} {} {:?}",
                    //         i, p1_turn, p1_river, turn, river, range1[i]
                    //     );
                    // }
                    // if (*j * 31 + p2_turn) * 30 + p2_river >= 52080 {
                    //     panic!(
                    //         "{} {} {} {} {} {:?}",
                    //         j, p2_turn, p2_river, turn, river, range2[*j]
                    //     );
                    // }
                    let board = [flop[0], flop[1], flop[2], turn, river];
                    let result = evaluate_winner((p1_one, p1_two), (p2_one, p2_two), &board);
                    let mut ev = [0.0; 2];
                    let mut reach = [1.0; 2];
                    unsafe {
                        update_regret(0, &buckets, result, &mut reach, 1.0, &mut ev, strat, g);
                    }
                    global_ev[0] += ev[0];
                    global_ev[1] += ev[1];
                    combos += 1;
                }
            }
        }
    }
    return (global_ev, combos);
}

fn train(
    g: &Game,
    strat_1: &mut [RegretStrategy; 2],
    strat_2: &mut [RegretStrategy; 2],
    strat_3: &mut [RegretStrategy; 2],
    strat_4: &mut [RegretStrategy; 2],
    flop: &[u8; 3],
    range1: &Vec<(u8, u8)>,
    range2: &Vec<(u8, u8)>,
) -> f64 {
    let start = Instant::now();
    // let combos = range1.len();
    let v = crossbeam::scope(|scope| {
        let a = scope.spawn(move |_| {
            return single_thread_train(g, strat_1, flop, range1, range2, 0, range1.len() / 4);
        });
        let b = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_2,
                flop,
                range1,
                range2,
                range1.len() / 4,
                range1.len() / 2,
            );
        });
        let c = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_3,
                flop,
                range1,
                range2,
                range1.len() / 2,
                range1.len() * 3 / 4,
            );
        });
        let d = single_thread_train(
            g,
            strat_4,
            flop,
            range1,
            range2,
            range1.len() * 3 / 4,
            range1.len(),
        );
        let a = a.join().unwrap();
        let b = b.join().unwrap();
        let c = c.join().unwrap();
        let combos = a.1 + b.1 + c.1 + d.1;
        let ev = [
            (a.0[0] + b.0[0] + c.0[0] + d.0[0]) / combos as f64,
            (a.0[1] + b.0[1] + c.0[1] + d.0[1]) / combos as f64,
        ];
        return ev;
    })
    .unwrap();
    println!("{:?}", v);
    return start.elapsed().as_secs_f64();
}

fn main() {
    // panic!("{} {}", internal, terminal);
    let g = Game::new(true, 1);
    println!("{:?}", g.transition[0]);
    let flop = [2, 15, 19];
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
    let range1 = gen_ranges(&combos, &flop);
    let range2 = gen_ranges(&combos, &flop);
    let mut safe_1 = SafeRegretStrategy::new(&g, 0, range1.len());
    let mut safe_2 = SafeRegretStrategy::new(&g, 1, range2.len());
    let regret_1 = RegretStrategy::new(
        &mut safe_1.regret,
        &mut safe_1.average_probability,
        &mut safe_1.updates,
    );
    let regret_2 = RegretStrategy::new(
        &mut safe_2.regret,
        &mut safe_2.average_probability,
        &mut safe_2.updates,
    );
    let mut strat_1 = [regret_1.0, regret_2.0];
    let mut strat_2 = [regret_1.1, regret_2.1];
    let mut strat_3 = [regret_1.2, regret_2.2];
    let mut strat_4 = [regret_1.3, regret_2.3];
    let mut total = 0.0;
    for i in 0..25 {
        let time = train(
            &g,
            &mut strat_1,
            &mut strat_2,
            &mut strat_3,
            &mut strat_4,
            &flop,
            &range1,
            &range2,
        );
        total += time;
        println!("{}: {}", i, time);
    }
    let mut i = 0;
    unsafe {
        for (c1, c2) in &range1 {
            let card1 = Card::from_u8(*c1);
            let card2 = Card::from_u8(*c2);
            println!(
                "P1 Open {}{}{}{}: {:?}",
                card1.value.to_char(),
                card1.suit.to_char(),
                card2.value.to_char(),
                card2.suit.to_char(),
                strat_1[0].get_average_normalized_probability(0, i, &g)
            );
            println!(
                "P2 vs. Check {}{}{}{}: {:?}",
                card1.value.to_char(),
                card1.suit.to_char(),
                card2.value.to_char(),
                card2.suit.to_char(),
                strat_1[1].get_average_normalized_probability(
                    g.transition[0][1].try_into().unwrap(),
                    i,
                    &g
                )
            );
            println!(
                "P2 vs. Bet {}{}{}{}: {:?}",
                card1.value.to_char(),
                card1.suit.to_char(),
                card2.value.to_char(),
                card2.suit.to_char(),
                strat_1[1].get_average_normalized_probability(
                    g.transition[0][0].try_into().unwrap(),
                    i,
                    &g
                )
            );
            i += 1;
            // let encoded: Vec<u8> = bincode::serialize(&strat.lock().unwrap()[0]).unwrap();
            // let mut file = File::create("test").unwrap();
            // file.write_all(&encoded).unwrap();
        }
    }
    println!("{}", total / 25.0);
    // println!("{}", safe_1.updates[0][55]);
    let mut best_resp_strat = SafeRegretStrategy::new(&g, 0, range1.len());
    let best_resp = BestResponse::new(0, &safe_2, &g, range1.clone(), range2.clone(), flop.clone());
    println!("computing br");
    let time = Instant::now();
    let val = best_resp.compute_best_response(0, &g, &mut best_resp_strat, None, None, None);
    println!("{} ({})", val, time.elapsed().as_secs_f64());

    let mut best_resp_strat = SafeRegretStrategy::new(&g, 1, range2.len());
    let best_resp = BestResponse::new(1, &safe_1, &g, range2.clone(), range1.clone(), flop.clone());
    let time = Instant::now();
    let val = best_resp.compute_best_response(0, &g, &mut best_resp_strat, None, None, None);
    println!("{} ({})", val, time.elapsed().as_secs_f64());
}
