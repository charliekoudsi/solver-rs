#![feature(const_mut_refs)]
mod best_response;
mod constants;
mod game;
mod regret;
use best_response::BestResponse;
use constants::{NO_DONK, NUM_CARDS};
use game::{evaluate_winner, get_buckets, Game};
use rand::{seq::SliceRandom, thread_rng};
use regret::{update_regret, RegretStrategy, SafeRegretStrategy};
use rs_poker::{gen_ranges, Card, Isomorph};
use std::{collections::HashMap, convert::TryInto, mem::transmute, time::Instant};

fn gen_range_cards(len: usize) -> Vec<usize> {
    let mut r = vec![0; len];
    for i in 0..len {
        r[i] = i;
    }
    return r;
}

fn single_thread_train(
    g: &Game,
    strat: &mut [RegretStrategy; 2],
    flop: &[u8; 3],
    range1: &Vec<(u8, u8)>,
    range2: &Vec<(u8, u8)>,
    min_i: usize,
    max_i: usize,
    map: &HashMap<((u8, u8), (u8, u8), u8, u8), i8>,
) -> ([f32; 2], usize) {
    let mut global_ev = [0.0; 2];
    let mut combos = 0;
    let mut rng = thread_rng();
    let mut turn_cards = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51,
    ];
    let mut river_cards = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51,
    ];
    let mut p2_range_cards = gen_range_cards(range2.len());
    for i in min_i..max_i {
        let (p1_one, p1_two) = range1[i];
        p2_range_cards.shuffle(&mut rng);
        for j in p2_range_cards.iter() {
            let (p2_one, p2_two) = range2[*j];
            if p1_one == p2_one || p1_one == p2_two || p1_two == p2_one || p1_two == p2_two {
                continue;
            }
            turn_cards.shuffle(&mut rng);
            for t in turn_cards.iter() {
                let turn = *t;
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
                for r in river_cards.iter() {
                    let river = *r;
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
                        [i, i * 47 + p1_turn, (i * 47 + p1_turn) * 46 + p1_river],
                        [*j, *j * 47 + p2_turn, (*j * 47 + p2_turn) * 46 + p2_river],
                    ];
                    let result = map.get(&((p1_one, p1_two), (p2_one, p2_two), turn, river));
                    if result == None {
                        panic!(
                            "({},{}) ({},{}) & {}, {}",
                            p1_one, p1_two, p2_one, p2_two, turn, river
                        );
                    }
                    let mut ev = [0.0; 2];
                    let mut reach = [1.0; 2];
                    unsafe {
                        update_regret(
                            0,
                            &buckets,
                            *result.unwrap(),
                            &mut reach,
                            1.0,
                            &mut ev,
                            strat,
                            g,
                        );
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

fn train_4(
    g: &Game,
    strat_1: &mut [RegretStrategy; 2],
    strat_2: &mut [RegretStrategy; 2],
    strat_3: &mut [RegretStrategy; 2],
    strat_4: &mut [RegretStrategy; 2],
    flop: &[u8; 3],
    range1: &Vec<(u8, u8)>,
    range2: &Vec<(u8, u8)>,
    map: &HashMap<((u8, u8), (u8, u8), u8, u8), i8>,
) -> (f32, [f32; 2]) {
    let start = Instant::now();
    // let combos = range1.len();
    let v = crossbeam::scope(|scope| {
        let a = scope.spawn(move |_| {
            return single_thread_train(g, strat_1, flop, range1, range2, 0, range1.len() / 4, map);
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
                map,
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
                map,
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
            map,
        );
        let a = a.join().unwrap();
        let b = b.join().unwrap();
        let c = c.join().unwrap();
        let combos = a.1 + b.1 + c.1 + d.1;
        let ev = [
            (a.0[0] + b.0[0] + c.0[0] + d.0[0]) / combos as f32,
            (a.0[1] + b.0[1] + c.0[1] + d.0[1]) / combos as f32,
        ];
        return ev;
    })
    .unwrap();
    return (start.elapsed().as_secs_f32(), v);
}

fn train_8(
    g: &Game,
    strat_1: &mut [RegretStrategy; 2],
    strat_2: &mut [RegretStrategy; 2],
    strat_3: &mut [RegretStrategy; 2],
    strat_4: &mut [RegretStrategy; 2],
    strat_5: &mut [RegretStrategy; 2],
    strat_6: &mut [RegretStrategy; 2],
    strat_7: &mut [RegretStrategy; 2],
    strat_8: &mut [RegretStrategy; 2],
    flop: &[u8; 3],
    range1: &Vec<(u8, u8)>,
    range2: &Vec<(u8, u8)>,
    map: &HashMap<((u8, u8), (u8, u8), u8, u8), i8>,
) -> (f32, [f32; 2]) {
    let start = Instant::now();
    // let combos = range1.len();
    let v = crossbeam::scope(|scope| {
        let a = scope.spawn(move |_| {
            return single_thread_train(g, strat_1, flop, range1, range2, 0, range1.len() / 4, map);
        });
        let b = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_2,
                flop,
                range1,
                range2,
                range1.len() / 8,
                range1.len() / 4,
                map,
            );
        });
        let c = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_3,
                flop,
                range1,
                range2,
                range1.len() / 4,
                range1.len() * 3 / 8,
                map,
            );
        });
        let d = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_4,
                flop,
                range1,
                range2,
                range1.len() * 3 / 8,
                range1.len() / 2,
                map,
            );
        });
        let e = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_5,
                flop,
                range1,
                range2,
                range1.len() / 2,
                range1.len() * 5 / 8,
                map,
            );
        });
        let f = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_6,
                flop,
                range1,
                range2,
                range1.len() * 5 / 8,
                range1.len() * 3 / 4,
                map,
            );
        });
        let g_t = scope.spawn(move |_| {
            return single_thread_train(
                g,
                strat_7,
                flop,
                range1,
                range2,
                range1.len() * 3 / 4,
                range1.len() * 7 / 8,
                map,
            );
        });
        let h = single_thread_train(
            g,
            strat_8,
            flop,
            range1,
            range2,
            range1.len() * 7 / 8,
            range1.len(),
            map,
        );
        let a = a.join().unwrap();
        let b = b.join().unwrap();
        let c = c.join().unwrap();
        let d = d.join().unwrap();
        let e = e.join().unwrap();
        let f = f.join().unwrap();
        let g_t = g_t.join().unwrap();
        let combos = a.1 + b.1 + c.1 + d.1 + e.1 + f.1 + g_t.1 + h.1;
        let ev = [
            (a.0[0] + b.0[0] + c.0[0] + d.0[0] + e.0[0] + f.0[0] + g_t.0[0] + h.0[0])
                / combos as f32,
            (a.0[1] + b.0[1] + c.0[1] + d.0[1] + e.0[1] + f.0[1] + g_t.0[1] + h.0[1])
                / combos as f32,
        ];
        return ev;
    })
    .unwrap();
    return (start.elapsed().as_secs_f32(), v);
}

fn main() {
    let g = Game::new();
    println!("{:?}", g.transition[0]);
    let flop = [18, 31, 35];
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
    let range2 = gen_ranges(&combos, &flop);
    let mut safe_1 = SafeRegretStrategy::new(&g, 0, range1.len());
    let mut safe_2 = SafeRegretStrategy::new(&g, 1, range2.len());
    // let regret_1 = RegretStrategy::new(
    //     &mut safe_1.regret,
    //     &mut safe_1.average_probability,
    //     &mut safe_1.updates,
    // );
    // let regret_2 = RegretStrategy::new(
    //     &mut safe_2.regret,
    //     &mut safe_2.average_probability,
    //     &mut safe_2.updates,
    // );
    // let mut strat_1 = [regret_1.0, regret_2.0];
    // let mut strat_2 = [regret_1.1, regret_2.1];
    // let mut strat_3 = [regret_1.2, regret_2.2];
    // let mut strat_4 = [regret_1.3, regret_2.3];
    let mut total = 0.0;
    let mut map: HashMap<((u8, u8), (u8, u8), u8, u8), i8> = HashMap::new();
    let mut board = [flop[0], flop[1], flop[2], 0, 0];
    for hand1 in range1.iter() {
        for hand2 in range2.iter() {
            if hand1.0 == hand2.0 || hand1.0 == hand2.1 || hand1.1 == hand2.0 || hand1.1 == hand2.1
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
                    map.insert((*hand1, *hand2, turn, river), winner);
                    map.insert((*hand2, *hand1, turn, river), -1 * winner);
                }
            }
        }
    }
    let mut dEV = 100.0;
    let mut runs = 0;
    // while dEV > 0.5 {
    let mut ev = 0.0;
    // {
    let regret_1 = RegretStrategy::new_8(
        &mut safe_1.regret,
        &mut safe_1.average_probability,
        &mut safe_1.updates,
    );
    let regret_2 = RegretStrategy::new_8(
        &mut safe_2.regret,
        &mut safe_2.average_probability,
        &mut safe_2.updates,
    );
    let mut strat_1 = [regret_1.0, regret_2.0];
    let mut strat_2 = [regret_1.1, regret_2.1];
    let mut strat_3 = [regret_1.2, regret_2.2];
    let mut strat_4 = [regret_1.3, regret_2.3];
    let mut strat_5 = [regret_1.4, regret_2.4];
    let mut strat_6 = [regret_1.5, regret_2.5];
    let mut strat_7 = [regret_1.6, regret_2.6];
    let mut strat_8 = [regret_1.7, regret_2.7];
    while total < 600.0 {
        let (time, run_ev) = train_8(
            &g,
            &mut strat_1,
            &mut strat_2,
            &mut strat_3,
            &mut strat_4,
            &mut strat_5,
            &mut strat_6,
            &mut strat_7,
            &mut strat_8,
            &flop,
            &range1,
            &range2,
            &map,
        );
        total += time;
        println!("{:?}", run_ev);
        println!("{}: {}", runs, time);
        ev = run_ev[0] + run_ev[1];
        runs += 1;
    }
    // for i in 0..10000 {
    // }
    // }
    let mut best_resp_strat = SafeRegretStrategy::new(&g, 0, range1.len());
    let best_resp = BestResponse::new(0, &safe_2, &g, range1.clone(), range2.clone(), flop.clone());
    println!("computing br");
    let time = Instant::now();
    let mut val =
        best_resp.compute_best_response(0, &g, &mut best_resp_strat, None, None, None, &map);
    // println!("{} ({})", val, time.elapsed().as_secs_f32());

    let mut best_resp_strat = SafeRegretStrategy::new(&g, 1, range2.len());
    let best_resp = BestResponse::new(1, &safe_1, &g, range2.clone(), range1.clone(), flop.clone());
    let new_val =
        best_resp.compute_best_response(0, &g, &mut best_resp_strat, None, None, None, &map);
    println!("[{}, {}]", val, new_val);
    val += new_val;
    // println!("{} ({})", val, time.elapsed().as_secs_f32());
    dEV = 100.0 * (val - ev) / val;
    println!("dEV: {}, Elapsed: {}", dEV, time.elapsed().as_secs_f32());
    // }
    let mut i = 0;
    for (c1, c2) in &range1 {
        let card1 = Card::from_u8(*c1);
        let card2 = Card::from_u8(*c2);
        println!(
            "P1 Open {}{}{}{}: {:?}",
            card1.value.to_char(),
            card1.suit.to_char(),
            card2.value.to_char(),
            card2.suit.to_char(),
            safe_1.get_average_normalized_probability(0, i, &g)
        );
        println!(
            "P2 vs. Check {}{}{}{}: {:?}",
            card1.value.to_char(),
            card1.suit.to_char(),
            card2.value.to_char(),
            card2.suit.to_char(),
            safe_2.get_average_normalized_probability(
                g.transition[0][1].try_into().unwrap(),
                i,
                &g
            )
        );
        if !NO_DONK {
            println!(
                "P2 vs. Bet {}{}{}{}: {:?}",
                card1.value.to_char(),
                card1.suit.to_char(),
                card2.value.to_char(),
                card2.suit.to_char(),
                safe_2.get_average_normalized_probability(
                    g.transition[0][0].try_into().unwrap(),
                    i,
                    &g
                )
            );
        }
        i += 1;
        // let encoded: Vec<u8> = bincode::serialize(&strat.lock().unwrap()[0]).unwrap();
        // let mut file = File::create("test").unwrap();
        // file.write_all(&encoded).unwrap();
    }
    println!("{}", total / runs as f32);
    // println!("{}", safe_1.updates[0][55]);
}
