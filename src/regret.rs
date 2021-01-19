use crate::game::{Game, NUM_INTERNAL};
use crossbeam_channel::Sender;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RegretStrategy {
    regret: Vec<Vec<[f64; 3]>>,
    pub average_probability: Vec<Vec<[f64; 3]>>,
}

impl RegretStrategy {
    pub fn new(player: usize, g: &Game, combos: usize) -> RegretStrategy {
        let mut regret = vec![vec![[0.0; 3]]; NUM_INTERNAL];
        // let mut regret: [Vec<[f64; 3]>; NUM_INTERNAL] = array_init::array_init(|_| vec![[0.0; 3]]);
        // let mut average_probability: [Vec<[f64; 3]>; NUM_INTERNAL] =
        //     array_init::array_init(|_| vec![[0.0; 3]]);
        let mut average_probability = vec![vec![[0.0; 3]]; NUM_INTERNAL];
        for i in 0..NUM_INTERNAL {
            if g.get_whose_turn(i) == player {
                let n;
                if g.get_round(i) == 0 {
                    n = combos;
                } else if g.get_round(i) == 1 {
                    n = combos * 31;
                } else {
                    n = combos * 31 * 30;
                }

                regret[i] = vec![[0.0; 3]; n];
                average_probability[i] = vec![[0.0; 3]; n];
            }
        }
        return RegretStrategy {
            regret,
            average_probability,
        };
    }

    #[inline(always)]
    fn get_regret(&mut self, u: usize, bucket: usize) -> &mut [f64; 3] {
        return &mut self.regret[u][bucket];
    }

    #[inline(always)]
    fn get_average_probability(&mut self, u: usize, bucket: usize) -> &mut [f64; 3] {
        return &mut self.average_probability[u][bucket];
    }

    #[inline]
    fn get_probability(&self, u: usize, bucket: usize, g: &Game) -> [f64; 3] {
        let mut probability = [0.0; 3];
        let mut regret_sum = 0.0;

        for i in 0..3 {
            regret_sum += self.regret[u][bucket][i].max(0.0);
        }

        if regret_sum > 1e-7 {
            for i in 0..3 {
                probability[i] = self.regret[u][bucket][i].max(0.0) / regret_sum;
            }
            return probability;
        }

        let p = 1.0 / (g.get_num_actions(u) as f64);

        for i in 0..3 {
            if g.can_do_action(i, u) {
                probability[i] = p;
            }
        }

        return probability;
    }

    #[inline]
    pub fn get_average_normalized_probability(
        &self,
        u: usize,
        bucket: usize,
        g: &Game,
    ) -> [f64; 3] {
        let mut prob_sum = 0.0;
        let mut probability = [0.0; 3];

        for i in 0..3 {
            prob_sum += self.average_probability[u][bucket][i];
        }

        if prob_sum > 1e-7 {
            for i in 0..3 {
                probability[i] = self.average_probability[u][bucket][i] / prob_sum;
            }
            return probability;
        }

        let p = 1.0 / (g.get_num_actions(u) as f64);
        for i in 0..3 {
            if g.can_do_action(i, u) {
                probability[i] = p;
            }
        }
        return probability;
    }

    #[inline(always)]
    fn update_avg_prob(&mut self, reach: f64, u: usize, bucket: usize, g: &Game) {
        let probability = self.get_probability(u, bucket, g);
        let avg_prob = self.get_average_probability(u, bucket);
        for i in 0..3 {
            avg_prob[i] += reach * probability[i];
        }
    }
}

pub fn update_regret(
    u: usize,
    buckets: &[[usize; 3]; 2],
    result: i8,
    reach: &mut [f64; 2],
    chance: f64,
    ev: &mut [f64; 2],
    cfr: &mut [f64; 2],
    strat: &mut [RegretStrategy; 2],
    // strat: Arc<Mutex<[RegretStrategy; 2]>>,
    g: &Game,
) {
    if g.is_terminal(u) {
        let amount = g.get_win_amount(u);

        if g.is_fold(u) {
            if g.who_folded(u) == 0 {
                ev[0] = -1.0 * (amount as f64) * reach[1] * chance;
                ev[1] = 1.0 * (amount as f64) * reach[0] * chance;
            } else {
                ev[0] = 1.0 * (amount as f64) * reach[1] * chance;
                ev[1] = -1.0 * (amount as f64) * reach[0] * chance;
            }
        } else {
            ev[0] = result as f64 * amount as f64 * reach[1] * chance;
            ev[1] = -1.0 * result as f64 * amount as f64 * reach[0] * chance;
        }
    } else if reach[0] < 1e-7 && reach[1] < 1e-7 {
        ev[0] = 0.0;
        ev[1] = 0.0;
    } else {
        let player = g.get_whose_turn(u);
        let opponent = 1 - player;
        let round = g.get_round(u);
        // {
        //     let mut guard = strat.lock().unwrap();
        //     let protected = &mut *guard;
        //     protected[player].update_avg_prob(reach[player], u, buckets[player][round], g);
        // }
        strat[player].update_avg_prob(reach[player], u, buckets[player][round], g);

        let mut util = 0.0;
        let mut regret_sum = 0.0;
        let old_reach = reach[player];
        let mut delta_regret = [0.0; 3];
        // let probability = {
        //     let mut guard = strat.lock().unwrap();
        //     let protected = &mut *guard;
        //     protected[player].get_probability(u, buckets[player][round], g)
        // };
        let probability = strat[player].get_probability(u, buckets[player][round], g);
        for i in 0..3 {
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
                    cfr,
                    strat,
                    g,
                );
                delta_regret[i] = ev[player];
                util += ev[player] * probability[i];
                regret_sum += ev[opponent];
            }
        }

        reach[player] = old_reach;

        // let mut regret = {
        //     let mut guard = strat.lock().unwrap();
        //     let protected = &mut *guard;
        //     protected[player]
        //         .get_regret(u, buckets[player][round])
        //         .to_owned()
        // };
        let regret = strat[player].get_regret(u, buckets[player][round]);
        for i in 0..3 {
            if g.can_do_action(i, u) {
                delta_regret[i] -= util;
                regret[i] += delta_regret[i];
                cfr[player] += delta_regret[i].max(0.0);
            }
        }
        ev[player] = util;
        ev[opponent] = regret_sum;
    }
}

// pub fn rewrite(
//     // u: usize,
//     // buckets: &[[usize; 3]; 2],
//     // result: i8,
//     // reach: &mut [f64; 2],
//     // chance: f64,
//     // ev: &mut [f64; 2],
//     // cfr: &mut [f64; 2],
//     // sender: Sender<(f64, usize, usize)>,
//     // strat: Arc<Mutex<[RegretStrategy; 2]>>,
//     g: &Game,
// ) {
//     let mut u = 0;
//     while !g.is_terminal(u) {
//         let player = g.get_whose_turn(u);
//         let opponent = 1 - player;
//         let round = g.get_round(u);
//     }
// }
