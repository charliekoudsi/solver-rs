use crate::constants::{NUM_INTERNAL, STARTING_POT, TOTAL_ACTIONS};
use crate::game::Game;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct SafeRegretStrategy {
    pub regret: Vec<Vec<[f32; TOTAL_ACTIONS]>>,
    pub average_probability: Vec<Vec<[f32; TOTAL_ACTIONS]>>,
    pub updates: Vec<Vec<usize>>,
}

impl SafeRegretStrategy {
    pub fn new(g: &Game, player: usize, combos: usize) -> Self {
        let mut regret = vec![vec![[0.0; TOTAL_ACTIONS]]; NUM_INTERNAL];
        let mut average_probability = vec![vec![[0.0; TOTAL_ACTIONS]]; NUM_INTERNAL];
        let mut updates = vec![vec![0]; NUM_INTERNAL];
        for i in 0..NUM_INTERNAL {
            if g.get_whose_turn(i) == player {
                let n;
                if g.get_round(i) == 0 {
                    n = combos;
                } else if g.get_round(i) == 1 {
                    n = combos * 47;
                } else {
                    n = combos * 47 * 46;
                }

                regret[i] = vec![[0.0; TOTAL_ACTIONS]; n];
                average_probability[i] = vec![[0.0; TOTAL_ACTIONS]; n];
                updates[i] = vec![0; n];
            }
        }
        return Self {
            regret,
            average_probability,
            updates,
        };
    }

    #[inline(always)]
    fn get_average_probability(&self, u: usize, bucket: usize) -> &[f32; TOTAL_ACTIONS] {
        return &self.average_probability[u][bucket];
    }

    #[inline]
    pub fn get_average_normalized_probability(
        &self,
        u: usize,
        bucket: usize,
        g: &Game,
    ) -> [f32; TOTAL_ACTIONS] {
        let mut prob_sum = 0.0;
        let mut probability = [0.0; TOTAL_ACTIONS];
        let average_probability = self.get_average_probability(u, bucket);
        for i in 0..TOTAL_ACTIONS {
            prob_sum += average_probability[i];
        }

        if prob_sum > 1e-7 {
            for i in 0..TOTAL_ACTIONS {
                probability[i] = average_probability[i] / prob_sum;
            }
            return probability;
        }

        let p = 1.0 / (g.get_num_actions(u) as f32);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                probability[i] = p;
            }
        }
        return probability;
    }
}

#[derive(Debug, Clone)]
pub struct RegretStrategy<'a> {
    regret: *mut Vec<[f32; TOTAL_ACTIONS]>,
    average_probability: *mut Vec<[f32; TOTAL_ACTIONS]>,
    updates: *mut Vec<usize>,
    lifetime: PhantomData<&'a f32>,
}

unsafe impl<'a> Send for RegretStrategy<'a> {}

impl<'a> RegretStrategy<'a> {
    pub fn new_4(
        regret: &'a mut Vec<Vec<[f32; TOTAL_ACTIONS]>>,
        average_probability: &'a mut Vec<Vec<[f32; TOTAL_ACTIONS]>>,
        updates: &'a mut Vec<Vec<usize>>,
    ) -> (
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
        RegretStrategy<'a>,
    ) {
        return (
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
        );
    }

    pub fn new_8(
        regret: &'a mut Vec<Vec<[f32; TOTAL_ACTIONS]>>,
        average_probability: &'a mut Vec<Vec<[f32; TOTAL_ACTIONS]>>,
        updates: &'a mut Vec<Vec<usize>>,
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
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
            RegretStrategy {
                regret: regret.as_mut_ptr(),
                average_probability: average_probability.as_mut_ptr(),
                updates: updates.as_mut_ptr(),
                lifetime: PhantomData,
            },
        );
    }

    #[inline(always)]
    unsafe fn get_regret(&mut self, u: usize, bucket: usize) -> &mut [f32; TOTAL_ACTIONS] {
        return &mut *((*self.regret.offset(u as isize))
            .as_mut_ptr()
            .offset(bucket as isize));
    }

    #[inline(always)]
    unsafe fn get_average_probability(
        &mut self,
        u: usize,
        bucket: usize,
    ) -> &mut [f32; TOTAL_ACTIONS] {
        return &mut *((*self.average_probability.offset(u as isize))
            .as_mut_ptr()
            .offset(bucket as isize));
    }

    // self does not to be mut
    #[inline]
    unsafe fn get_probability(
        &mut self,
        u: usize,
        bucket: usize,
        g: &Game,
    ) -> [f32; TOTAL_ACTIONS] {
        let mut probability = [0.0; TOTAL_ACTIONS];
        let mut regret_sum = 0.0;
        let regret = self.get_regret(u, bucket);
        for i in 0..TOTAL_ACTIONS {
            regret_sum += regret[i].max(0.0);
        }

        if regret_sum > 1e-7 {
            for i in 0..TOTAL_ACTIONS {
                probability[i] = regret[i].max(0.0) / regret_sum;
            }
            return probability;
        }

        let p = 1.0 / (g.get_num_actions(u) as f32);

        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                probability[i] = p;
            }
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
    ) -> [f32; TOTAL_ACTIONS] {
        let mut prob_sum = 0.0;
        let mut probability = [0.0; TOTAL_ACTIONS];
        let average_probability = self.get_average_probability(u, bucket);
        for i in 0..TOTAL_ACTIONS {
            prob_sum += average_probability[i];
        }

        if prob_sum > 1e-7 {
            for i in 0..TOTAL_ACTIONS {
                probability[i] = average_probability[i] / prob_sum;
            }
            return probability;
        }

        let p = 1.0 / (g.get_num_actions(u) as f32);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                probability[i] = p;
            }
        }
        return probability;
    }

    #[inline(always)]
    unsafe fn update_avg_prob(&mut self, reach: f32, u: usize, bucket: usize, g: &Game) {
        let probability = self.get_probability(u, bucket, g);
        let avg_prob = self.get_average_probability(u, bucket);
        for i in 0..TOTAL_ACTIONS {
            avg_prob[i] += reach * probability[i];
        }
    }
}

pub unsafe fn update_regret(
    u: usize,
    buckets: &[[usize; 3]; 2],
    result: i8,
    reach: &mut [f32; 2],
    chance: f32,
    ev: &mut [f32; 2],
    strat: &mut [RegretStrategy; 2],
    g: &Game,
) {
    if g.is_terminal(u) {
        let amount = g.get_win_amount(u);

        if g.is_fold(u) {
            if g.who_folded(u) == 0 {
                ev[0] = -1.0 * (amount - STARTING_POT) as f32 * reach[1] * chance;
                ev[1] = 1.0 * (amount as f32) * reach[0] * chance;
            } else {
                ev[0] = 1.0 * (amount as f32) * reach[1] * chance;
                ev[1] = -1.0 * (amount - STARTING_POT) as f32 * reach[0] * chance;
            }
        } else {
            if result == 1 {
                ev[0] = result as f32 * amount as f32 * reach[1] * chance;
                ev[1] = -1.0 * result as f32 * (amount - STARTING_POT) as f32 * reach[0] * chance;
            } else if result == -1 {
                ev[0] = result as f32 * (amount - STARTING_POT) as f32 * reach[1] * chance;
                ev[1] = -1.0 * result as f32 * amount as f32 * reach[0] * chance;
            } else {
                ev[0] = STARTING_POT as f32 / 2.0 * reach[1] * chance;
                ev[1] = STARTING_POT as f32 / 2.0 * reach[0] * chance;
            }
        }
    } else {
        let player = g.get_whose_turn(u);
        let opponent = 1 - player;
        let round = g.get_round(u);
        strat[player].update_avg_prob(reach[player], u, buckets[player][round], g);

        let mut util = 0.0;
        let mut regret_sum = 0.0;
        let old_reach = reach[player];
        let mut delta_regret = [0.0; TOTAL_ACTIONS];
        let probability = strat[player].get_probability(u, buckets[player][round], g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                reach[player] = old_reach * probability[i];
                // let strategy = Arc::clone(&strat);
                update_regret(
                    g.do_action(i, u) as usize,
                    buckets,
                    result,
                    reach,
                    chance,
                    ev,
                    strat,
                    g,
                );
                delta_regret[i] = ev[player];
                util += ev[player] * probability[i];
                regret_sum += ev[opponent];
            }
        }

        reach[player] = old_reach;
        let regret = strat[player].get_regret(u, buckets[player][round]);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                delta_regret[i] -= util;
                regret[i] += delta_regret[i];
            }
        }
        ev[player] = util;
        ev[opponent] = regret_sum;
    }
}
