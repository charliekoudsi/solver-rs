use crate::constants::{Array1, Array2, COMBOS, NUM_INTERNAL, STARTING_POT, TOTAL_ACTIONS};
use crate::game::Game;
use crate::terminal::{eval_fold, eval_showdown, rank_board, RankedArray};
use crossbeam_utils::thread as crossbeam;
use std::{
    marker::PhantomData,
    ops::{AddAssign, Div, Mul},
};

#[derive(Debug)]
pub struct SafeRegretStrategy {
    pub regret: Vec<Vec<Array2>>,
    pub average_probability: Vec<Vec<Array2>>,
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
                    regret[i].push(Array2::zeros());
                    average_probability[i].push(Array2::zeros());
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
    fn get_average_probability(&self, u: usize, bucket: usize) -> &Array2 {
        return &self.average_probability[u][bucket];
    }

    #[inline]
    pub fn get_average_normalized_probability(&self, u: usize, bucket: usize, g: &Game) -> Array2 {
        let mut probability = Array2::zeros();
        let average_probability = self.get_average_probability(u, bucket);
        // let prob_sum = average_probability.sum_axis(Axis(0));
        let prob_sum = average_probability.column_sum();

        let p = 1.0 / (g.get_num_actions(u) as f32);
        for i in 0..TOTAL_ACTIONS {
            probability.column_mut(i).zip_zip_apply(
                &average_probability.column(i),
                &prob_sum,
                |_, avg, p_sum| {
                    if p_sum > 1e-7 {
                        avg / p_sum
                    } else if g.can_do_action(i, u) {
                        p
                    } else {
                        0.0
                    }
                },
            );
        }
        return probability;
    }
}

#[derive(Debug, Clone)]
pub struct RegretStrategy<'a> {
    regret: *mut Vec<Array2>,
    average_probability: *mut Vec<Array2>,
    lifetime: PhantomData<&'a f32>,
}

unsafe impl<'a> Send for RegretStrategy<'a> {}

impl<'a> RegretStrategy<'a> {
    pub fn new(
        regret1: &'a mut Vec<Vec<Array2>>,
        average_probability1: &'a mut Vec<Vec<Array2>>,
        regret2: &'a mut Vec<Vec<Array2>>,
        average_probability2: &'a mut Vec<Vec<Array2>>,
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
        regret: &'a mut Vec<Vec<Array2>>,
        average_probability: &'a mut Vec<Vec<Array2>>,
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
    pub unsafe fn get_regret(&mut self, u: usize, bucket: usize) -> &mut Array2 {
        return &mut (*(self.regret.offset(u as isize)))[bucket];
    }

    #[inline(always)]
    unsafe fn get_average_probability(&mut self, u: usize, bucket: usize) -> &mut Array2 {
        return &mut (*(self.average_probability.offset(u as isize)))[bucket];
    }

    // self does not to be mut
    #[inline]
    unsafe fn get_probability(&mut self, u: usize, bucket: usize, g: &Game) -> Array2 {
        let mut probability = Array2::zeros();
        let regret = self.get_regret(u, bucket);
        let mapped_regret = regret.map(|x| x.max(0.0));
        let regret_sum = mapped_regret.column_sum();
        let p = 1.0 / (g.get_num_actions(u) as f32);

        for i in 0..TOTAL_ACTIONS {
            probability.column_mut(i).zip_zip_apply(
                &mapped_regret.column(i),
                &regret_sum,
                |_, r, r_sum| {
                    if r_sum > 1e-7 {
                        r / r_sum
                    } else if g.can_do_action(i, u) {
                        p
                    } else {
                        0.0
                    }
                },
            );
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
    ) -> Array2 {
        let mut probability = Array2::zeros();
        let average_probability = self.get_average_probability(u, bucket);
        let prob_sum = average_probability.column_sum();
        let p = 1.0 / (g.get_num_actions(u) as f32);

        // for i in 0..TOTAL_ACTIONS {
        //     probability.column_mut(i).zip_zip_apply(
        //         &mapped_regret.column(i),
        //         &regret_sum,
        //         |_, r, r_sum| {
        //             if r_sum > 1e-7 {
        //                 r / r_sum
        //             } else {
        //                 p
        //             }
        //         },
        //     );
        // }

        for i in 0..TOTAL_ACTIONS {
            probability.column_mut(i).zip_zip_apply(
                &average_probability.column(i),
                &prob_sum,
                |_, avg, p_sum| {
                    if p_sum > 1e-7 {
                        avg / p_sum
                    } else if g.can_do_action(i, u) {
                        p
                    } else {
                        0.0
                    }
                },
            );
        }
        return probability;
    }

    #[inline(always)]
    unsafe fn update_avg_prob(&mut self, reach: &Array1, u: usize, bucket: usize, g: &Game) {
        let mut probability = self.get_probability(u, bucket, g);
        let avg_prob = self.get_average_probability(u, bucket);

        // This is really bad:
        // TODO: Anything but this
        for i in 0..TOTAL_ACTIONS {
            probability.column_mut(i).component_mul(reach);
        }

        (*avg_prob).add_assign(probability);
    }
}

pub unsafe fn update_regret(
    u: usize,
    buckets: &[[usize; 3]; 2],
    ranked: &RankedArray,
    reach: &mut [Array1; 2],
    chance: &[Array1; 2],
    ev: &mut [Array1; 2],
    strat: &mut [RegretStrategy; 2],
    g: &Game,
) {
    if g.is_terminal(u) {
        let amount = g.get_win_amount(u);

        if g.is_fold(u) {
            if g.who_folded(u) == 0 {
                ev[0] = eval_fold(-1.0 * (amount - STARTING_POT) as f32, &reach[1])
                    .component_mul(&chance[0]);
                ev[1] = eval_fold(1.0 * amount as f32, &reach[0]).component_mul(&chance[1]);
            } else {
                ev[0] = eval_fold(1.0 * amount as f32, &reach[1]).component_mul(&chance[0]);
                ev[1] = eval_fold(-1.0 * (amount - STARTING_POT) as f32, &reach[0])
                    .component_mul(&chance[1]);
            }
        } else {
            ev[0] = eval_showdown(amount as f32, ranked, &reach[1]).component_mul(&chance[0]);
            ev[1] = eval_showdown(amount as f32, ranked, &reach[0]).component_mul(&chance[1]);
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

        let mut util = Array1::zeros();
        let mut regret_sum = Array1::zeros();
        let old_reach = reach[player].clone();
        // This will break if TOTAL_ACTIONS != 3
        let mut delta_regret = [Array1::zeros(), Array1::zeros(), Array1::zeros()];
        let probability = strat[player].get_probability(u, buckets[player][round], g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                reach[player] = probability.column(i).component_mul(&old_reach);
                // Zip::from(&mut reach[player])
                //     .and(probability.row(i))
                //     .and(&old_reach)
                //     .for_each(|r, &p, &old| {
                //         *r = old * p;
                //     });
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
                delta_regret[i] += &ev[player];
                util += ev[player].component_mul(&probability.column(i));
                regret_sum += ev[opponent];
            }
        }

        reach[player] = old_reach;
        let regret = strat[player].get_regret(u, buckets[player][round]);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                delta_regret[i] -= util;
                regret.column_mut(i).add_assign(&delta_regret[i]);
                // Zip::from(regret.row_mut(i))
                //     .and(&delta_regret[i])
                //     .for_each(|r, &d| {
                //         *r += d;
                //     });
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
    chance: &[Array1; 2],
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
    chance: &[Array1; 2],
    strat: &mut [RegretStrategy; 2],
    g: &Game,
    min_index: u8,
    max_index: u8,
) {
    let mut board = [flop[0], flop[1], flop[2], 0, 0];
    let mut num_iters = 0;
    let mut global_p0 = Array1::zeros();
    let mut global_p1 = Array1::zeros();
    for t in min_index..max_index {
        if t != board[0] && t != board[1] && t != board[2] {
            board[3] = t;
            for r in 0..52 {
                if r != board[0] && r != board[1] && r != board[2] && r != t {
                    board[4] = r;
                    let ranked = rank_board(&board);
                    let mut p0_ev = Array1::zeros();
                    let mut p1_ev = Array1::zeros();
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
}
