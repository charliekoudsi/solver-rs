use crate::constants::{COMBOS, NUM_INTERNAL, STARTING_POT, TOTAL_ACTIONS};
use crate::game::Game;
use crate::terminal::{eval_fold, eval_showdown, rank_board, RankedHand};
use crossbeam_utils::thread as crossbeam;
use ndarray::{Array1, Array2, Axis, Zip};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct SafeRegretStrategy {
    pub regret: Vec<Vec<Array2<f32>>>,
    pub average_probability: Vec<Vec<Array2<f32>>>,
}

impl SafeRegretStrategy {
    pub fn new(g: &Game, player: usize) -> Self {
        let mut regret = Vec::with_capacity(NUM_INTERNAL);
        let mut average_probability = Vec::with_capacity(NUM_INTERNAL);
        for i in 0..NUM_INTERNAL {
            if g.get_whose_turn(i) == player {
                let n;
                if g.get_round(i) == 0 {
                    n = 1;
                } else if g.get_round(i) == 1 {
                    n = 49;
                } else {
                    n = 49 * 48;
                }
                regret.push(Vec::with_capacity(n));
                average_probability.push(Vec::with_capacity(n));
                for _ in 0..n {
                    regret[i].push(Array2::<f32>::zeros((TOTAL_ACTIONS, COMBOS)));
                    average_probability[i].push(Array2::<f32>::zeros((TOTAL_ACTIONS, COMBOS)));
                }
            } else {
                regret.push(Vec::with_capacity(0));
                average_probability.push(Vec::with_capacity(0));
            }
        }
        return Self {
            regret,
            average_probability,
        };
    }

    #[inline(always)]
    fn get_average_probability(&self, u: usize, bucket: usize) -> &Array2<f32> {
        return &self.average_probability[u][bucket];
    }

    #[inline]
    pub fn get_average_normalized_probability(
        &self,
        u: usize,
        bucket: usize,
        g: &Game,
    ) -> Array2<f32> {
        let mut probability = Array2::<f32>::zeros((TOTAL_ACTIONS, COMBOS));
        let average_probability = self.get_average_probability(u, bucket);
        let prob_sum = average_probability.sum_axis(Axis(0));

        let p = 1.0 / (g.get_num_actions(u) as f32);
        for i in 0..TOTAL_ACTIONS {
            Zip::from(probability.row_mut(i))
                .and(&prob_sum)
                .and(&average_probability.row(i))
                .for_each(|prob, &p_sum, &avg| {
                    if p_sum > 1e-7 {
                        *prob = avg / p_sum;
                    } else {
                        if g.can_do_action(i, u) {
                            *prob = p;
                        }
                    }
                });
        }
        return probability;
    }
}

#[derive(Debug, Clone)]
pub struct RegretStrategy<'a> {
    regret: *mut Vec<Array2<f32>>,
    average_probability: *mut Vec<Array2<f32>>,
    lifetime: PhantomData<&'a f32>,
}

unsafe impl<'a> Send for RegretStrategy<'a> {}

impl<'a> RegretStrategy<'a> {
    pub fn new(
        regret1: &'a mut Vec<Vec<Array2<f32>>>,
        average_probability1: &'a mut Vec<Vec<Array2<f32>>>,
        regret2: &'a mut Vec<Vec<Array2<f32>>>,
        average_probability2: &'a mut Vec<Vec<Array2<f32>>>,
        len: usize,
    ) -> Vec<[RegretStrategy<'a>; 2]> {
        let mut strategies = Vec::with_capacity(len);
        for _ in 0..len {
            strategies.push([
                RegretStrategy {
                    regret: regret1.as_mut_ptr(),
                    average_probability: average_probability1.as_mut_ptr(),
                    lifetime: PhantomData,
                },
                RegretStrategy {
                    regret: regret2.as_mut_ptr(),
                    average_probability: average_probability2.as_mut_ptr(),
                    lifetime: PhantomData,
                },
            ])
        }

        return strategies;
    }

    pub fn new_8(
        regret: &'a mut Vec<Vec<Array2<f32>>>,
        average_probability: &'a mut Vec<Vec<Array2<f32>>>,
    ) -> (
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
    ) {
        return (
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                lifetime: PhantomData,
            },
        );
    }

    #[inline(always)]
    pub unsafe fn get_regret(&mut self, u: usize, bucket: usize) -> &mut Array2<f32> {
        return &mut (*(self.regret.offset(u as isize)))[bucket];
    }

    #[inline(always)]
    unsafe fn get_average_probability(&mut self, u: usize, bucket: usize) -> &mut Array2<f32> {
        return &mut (*(self.average_probability.offset(u as isize)))[bucket];
    }

    // self does not to be mut
    #[inline]
    unsafe fn get_probability(&mut self, u: usize, bucket: usize, g: &Game) -> Array2<f32> {
        let mut probability = Array2::<f32>::zeros((TOTAL_ACTIONS, COMBOS));
        let regret = self.get_regret(u, bucket);
        let regret_sum = regret.mapv(|x| x.max(0.0)).sum_axis(Axis(0));
        let p = 1.0 / (g.get_num_actions(u) as f32);

        for i in 0..TOTAL_ACTIONS {
            Zip::from(probability.row_mut(i))
                .and(&regret_sum)
                .and(regret.row(i))
                .for_each(|prob, &r_sum, &r| {
                    if r_sum > 1e-7 {
                        *prob = r.max(0.0) / r_sum;
                    } else {
                        if g.can_do_action(i, u) {
                            *prob = p;
                        }
                    }
                });
        }
        return probability;
    }

    // self does not to be mut
    #[inline]
    pub unsafe fn get_average_normalized_probability(
        &mut self,
        u: usize,
        bucket: usize,
        g: &Game,
    ) -> Array2<f32> {
        let mut probability = Array2::<f32>::zeros((TOTAL_ACTIONS, COMBOS));
        let average_probability = self.get_average_probability(u, bucket);
        let prob_sum = average_probability.sum_axis(Axis(0));

        let p = 1.0 / (g.get_num_actions(u) as f32);
        for i in 0..TOTAL_ACTIONS {
            Zip::from(probability.row_mut(i))
                .and(&prob_sum)
                .and(&average_probability.row(i))
                .for_each(|prob, &p_sum, &avg| {
                    if p_sum > 1e-7 {
                        *prob = avg / p_sum;
                    } else {
                        if g.can_do_action(i, u) {
                            *prob = p;
                        }
                    }
                });
        }
        return probability;
    }

    #[inline(always)]
    unsafe fn update_avg_prob(&mut self, reach: &Array1<f32>, u: usize, bucket: usize, g: &Game) {
        let probability = self.get_probability(u, bucket, g);
        let avg_prob = self.get_average_probability(u, bucket);
        *avg_prob += &(probability * reach);
    }
}

pub unsafe fn update_regret(
    u: usize,
    buckets: &[[usize; 3]; 2],
    ranked: &Array1<RankedHand>,
    reach: &mut [Array1<f32>; 2],
    chance: &[Array1<f32>; 2],
    ev: &mut [Array1<f32>; 2],
    strat: &mut [RegretStrategy; 2],
    g: &Game,
) {
    if g.is_terminal(u) {
        let amount = g.get_win_amount(u);

        if g.is_fold(u) {
            if g.who_folded(u) == 0 {
                ev[0] = &eval_fold(-1.0 * (amount - STARTING_POT) as f32, &reach[1]) * &chance[0];
                ev[1] = &eval_fold(1.0 * amount as f32, &reach[0]) * &chance[1];
            } else {
                ev[0] = &eval_fold(1.0 * amount as f32, &reach[1]) * &chance[0];
                ev[1] = &eval_fold(-1.0 * (amount - STARTING_POT) as f32, &reach[0]) * &chance[1];
            }
        } else {
            ev[0] = &eval_showdown(amount as f32, ranked, &reach[1]) * &chance[0];
            ev[1] = &eval_showdown(amount as f32, ranked, &reach[0]) * &chance[1];
            // println!("{}", ev[0].sum());
            // println!("{}", ev[1].sum());
            // println!(
            //     "{:?}",
            //     eval_showdown(amount as f32, ranked, reach[1]) * chance[0]
            // );
            // panic!("done");
        }
    }
    //      This (below) has a massive performance impact (~5-10x) in non-vectorized implementation
    //      I have not yet tested this in the vectorized version
    // else if reach[0] < 1e-15 && reach[1] < 1e-15 {
    //     ev[0] = 0.0;
    //     ev[1] = 0.0;
    // }
    else {
        let player = g.get_whose_turn(u);
        let opponent = 1 - player;
        let round = g.get_round(u);
        strat[player].update_avg_prob(&reach[player], u, buckets[player][round], g);

        let mut util = Array1::<f32>::zeros(COMBOS);
        let mut regret_sum = Array1::<f32>::zeros(COMBOS);
        let old_reach = reach[player].clone();
        // This will break if TOTAL_ACTIONS != 3
        let mut delta_regret = [
            Array1::<f32>::zeros(COMBOS),
            Array1::<f32>::zeros(COMBOS),
            Array1::<f32>::zeros(COMBOS),
        ];
        let probability = strat[player].get_probability(u, buckets[player][round], g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                Zip::from(&mut reach[player])
                    .and(probability.row(i))
                    .and(&old_reach)
                    .for_each(|r, &p, &old| {
                        *r = old * p;
                    });
                update_regret(
                    g.do_action(i, u) as usize,
                    buckets,
                    ranked,
                    reach,
                    chance,
                    ev,
                    strat,
                    g,
                );
                delta_regret[i] = &delta_regret[i] + &ev[player];
                util = util + &ev[player] * &(probability.row(i));
                regret_sum = regret_sum + ev[opponent].clone();
            }
        }

        reach[player] = old_reach;
        let regret = strat[player].get_regret(u, buckets[player][round]);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                delta_regret[i] -= &util;
                Zip::from(regret.row_mut(i))
                    .and(&delta_regret[i])
                    .for_each(|r, &d| {
                        *r += d;
                    });
            }
        }
        ev[player] = util;
        ev[opponent] = regret_sum;
    }
}

const fn get_buckets(board: &[u8; 5]) -> (usize, usize) {
    let mut turn_bucket = board[3];
    if turn_bucket > board[2] {
        turn_bucket -= 3;
    } else if turn_bucket > board[1] {
        turn_bucket -= 2;
    } else if turn_bucket > board[0] {
        turn_bucket -= 1;
    }

    let mut river_bucket = board[4];
    if river_bucket > board[2] {
        river_bucket -= 3;
    } else if river_bucket > board[1] {
        river_bucket -= 2;
    } else if river_bucket > board[0] {
        river_bucket -= 1;
    }

    if board[4] > board[3] {
        river_bucket -= 1;
    }
    let turn_bucket = turn_bucket as usize;
    let river_bucket = river_bucket as usize;
    return (turn_bucket as usize, 48 * turn_bucket + river_bucket);
}

pub fn train(
    flop: &[u8; 3],
    chance: &[Array1<f32>; 2],
    strategies: &mut [[RegretStrategy; 2]],
    g: &Game,
) {
    let len = strategies.len();
    crossbeam::scope(|scope| {
        for (i, strat) in strategies.iter_mut().enumerate() {
            let start_index = (i * 52) / len;
            let end_index = ((i + 1) * 52) / len;
            scope.spawn(move |_| {
                single_thread_train(flop, chance, strat, g, start_index as u8, end_index as u8);
            });
        }
    })
    .unwrap();
}

fn single_thread_train(
    flop: &[u8; 3],
    chance: &[Array1<f32>; 2],
    strat: &mut [RegretStrategy; 2],
    g: &Game,
    min_index: u8,
    max_index: u8,
) {
    let mut board = [flop[0], flop[1], flop[2], 0, 0];
    let mut num_iters = 0;
    let mut global_p0 = Array1::<f32>::zeros(COMBOS);
    let mut global_p1 = Array1::<f32>::zeros(COMBOS);
    for t in min_index..max_index {
        if t != board[0] && t != board[1] && t != board[2] {
            board[3] = t;
            for r in 0..52 {
                if r != board[0] && r != board[1] && r != board[2] && r != t {
                    board[4] = r;
                    let ranked = rank_board(&board);
                    let mut p0_ev = Array1::<f32>::zeros(COMBOS);
                    let mut p1_ev = Array1::<f32>::zeros(COMBOS);
                    let mut p0_reach = chance[0].clone();
                    let mut p1_reach = chance[1].clone();
                    // p0_reach *= chance[0];
                    // p1_reach *= chance[1];
                    let mut ev = [p0_ev, p1_ev];
                    let mut reach = [p0_reach, p1_reach];
                    let (t_bucket, r_bucket) = get_buckets(&board);
                    let buckets = [[0, t_bucket, r_bucket], [0, t_bucket, r_bucket]];
                    unsafe {
                        update_regret(0, &buckets, &ranked, &mut reach, chance, &mut ev, strat, g);
                    }
                    num_iters += 1;
                    global_p0 = global_p0 + &ev[0];
                    global_p1 = global_p1 + &ev[1];
                }
            }
        }
    }
    println!(
        "{:?}, {:?}",
        global_p0 / num_iters as f32,
        global_p1 / num_iters as f32
    );
}
