use crate::constants::{
    Array1, Array2, NUM_CARDS, NUM_INTERNAL, NUM_TERMINAL, STARTING_POT, TOTAL_ACTIONS,
};
use crate::game::{get_bucket, get_turn_bucket, Game};
use crate::regret::SafeRegretStrategy as RegretStrategy;
use crate::terminal::{eval_fold, eval_showdown, rank_board, RankedArray};
use crate::winners::Winners;
use std::{collections::HashMap, ops::Div};

type TerminalProb = Vec<Vec<Array1>>;
type Range = Vec<(u8, u8)>;

pub struct BestResponse {
    player: usize,
    pub terminal_probs: TerminalProb,
    range: Range,
    opp_range: Range,
    opp_combos: Array1,
    flop: [u8; 3],
}

fn compute_terminal_probabilities(
    player: usize,
    flop: &[u8; 3],
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    let reach = Array1::repeat(1.0);
    compute_flop_probabilities(0, player, flop, &reach, strat, prob, g);
}

fn compute_flop_probabilities(
    u: usize,
    player: usize,
    flop: &[u8; 3],
    reach: &Array1,
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    if g.is_terminal(u) {
        prob[u - NUM_INTERNAL][0] = *reach;
    } else if g.get_round(u) == 1 {
        let mut turn = 0;
        for i in 0..NUM_CARDS {
            if flop[0] == i || flop[1] == i || flop[2] == i {
                continue;
            }
            compute_turn_probabilities(u, player, flop, turn, reach, strat, prob, g);
            turn += 1;
        }
    } else if g.get_whose_turn(u) == player {
        let actions = strat.get_average_normalized_probability(u, 0, g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_flop_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    &(reach.component_mul(&actions.column(i))),
                    strat,
                    prob,
                    g,
                );
            }
        }
    } else {
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_flop_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    reach,
                    strat,
                    prob,
                    g,
                );
            }
        }
    }
}

fn compute_turn_probabilities(
    u: usize,
    player: usize,
    flop: &[u8; 3],
    turn: u8,
    reach: &Array1,
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    if g.is_terminal(u) {
        prob[u - NUM_INTERNAL][turn as usize] = *reach;
    } else if g.get_round(u) == 2 {
        let mut river = 0;
        for i in 0..NUM_CARDS {
            if flop[0] == i || flop[1] == i || flop[2] == i || turn == i {
                continue;
            }
            compute_river_probabilities(u, player, flop, turn, river, reach, strat, prob, g);
            river += 1;
        }
    } else if g.get_whose_turn(u) == player {
        let actions = strat.get_average_normalized_probability(u, turn as usize, g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_turn_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    turn,
                    &(reach.component_mul(&actions.column(i))),
                    strat,
                    prob,
                    g,
                );
            }
        }
    } else {
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_turn_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    turn,
                    reach,
                    strat,
                    prob,
                    g,
                );
            }
        }
    }
}

fn compute_river_probabilities(
    u: usize,
    player: usize,
    flop: &[u8; 3],
    turn: u8,
    river: u8,
    reach: &Array1,
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    if g.is_terminal(u) {
        prob[u - NUM_INTERNAL][turn as usize * 48 + river as usize] = *reach;
    } else if g.get_whose_turn(u) == player {
        let actions =
            strat.get_average_normalized_probability(u, turn as usize * 48 + river as usize, g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_river_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    turn,
                    river,
                    &(reach.component_mul(&actions.column(i))),
                    strat,
                    prob,
                    g,
                )
            }
        }
    } else {
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_river_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    turn,
                    river,
                    reach,
                    strat,
                    prob,
                    g,
                )
            }
        }
    }
}

fn compute_opp_combos(range: &Range, opp_range: &Range) -> Array1 {
    let mut combos = Array1::zeros();
    for i in 0..range.len() {
        let mut opp_combos = 0.0;
        let (card1, card2) = range[i];
        for (opp1, opp2) in opp_range.iter() {
            if *opp1 != card1 && *opp1 != card2 && *opp2 != card1 && *opp2 != card2 {
                opp_combos += 1.0;
            }
        }
        combos[i] = opp_combos;
    }
    return combos;
}

impl BestResponse {
    pub fn new(
        p: usize,
        strat: &RegretStrategy,
        g: &Game,
        range: Range,
        opp_range: Range,
        flop: [u8; 3],
    ) -> Self {
        let mut probability: TerminalProb = vec![Vec::with_capacity(1); NUM_TERMINAL];
        for i in 0..NUM_TERMINAL {
            let n = {
                if g.get_round(i + NUM_INTERNAL) == 0 {
                    1
                } else if g.get_round(i + NUM_INTERNAL) == 1 {
                    49
                } else {
                    49 * 48
                }
            };
            for _ in 0..n {
                probability[i].push(Array1::zeros());
            }
        }
        compute_terminal_probabilities(1 - p, &flop, strat, &mut probability, g);
        let opp_combos = compute_opp_combos(&range, &opp_range);
        return Self {
            player: p,
            terminal_probs: probability,
            range,
            opp_range,
            opp_combos,
            flop,
        };
    }

    pub fn compute_chance_prob(&self, round: usize) -> Array1 {
        let combos = self.range.len() as f64;
        let opp_combos = self.opp_combos;
        if round == 0 {
            return Array1::repeat(1.0).component_div(&(combos * opp_combos));
        } else if round == 1 {
            return Array1::repeat(1.0).component_div(&(combos * opp_combos * 48.0));
        }
        return Array1::repeat(1.0).component_div(&(combos * opp_combos * 48.0 * 47.0));
    }

    pub fn compute_best_response(&self, g: &Game, strat: &mut RegretStrategy) -> f64 {
        let mut cards = [0; 5];
        cards[0] = self.flop[0];
        cards[1] = self.flop[1];
        cards[2] = self.flop[2];
        let mut turn = 0;
        let mut ev = 0.0;
        for i in 0..NUM_CARDS {
            if i == cards[0] || i == cards[1] || i == cards[2] {
                continue;
            }
            let mut river = 0;
            cards[3] = i;
            for j in 0..NUM_CARDS {
                if j == cards[0] || j == cards[1] || j == cards[2] || j == i {
                    continue;
                }
                cards[4] = j;
                let hands = rank_board(&cards);
                ev += self
                    .compute_best_response_2(0, g, strat, turn, river, &hands)
                    .component_mul(&self.compute_chance_prob(0))
                    .sum();
                river += 1;
            }
            turn += 1;
        }
        return ev;
    }

    fn compute_best_response_2(
        &self,
        u: usize,
        g: &Game,
        strat: &mut RegretStrategy,
        turn: usize,
        river: usize,
        hands: &RankedArray,
    ) -> Array1 {
        if g.is_terminal(u) {
            if g.is_showdown(u) {
                let chance = self.compute_chance_prob(2);
                let p = {
                    if g.get_round(u) == 2 {
                        self.terminal_probs[u - NUM_INTERNAL][turn * 48 + river]
                    } else if g.get_round(u) == 1 {
                        self.terminal_probs[u - NUM_INTERNAL][turn]
                    } else {
                        self.terminal_probs[u - NUM_INTERNAL][0]
                    }
                };
                let ev =
                    eval_showdown(g.get_win_amount(u) as f64, hands, &p).component_mul(&chance);

                return ev;
            } else {
                let round = g.get_round(u);
                let bucket = {
                    if round == 0 {
                        0
                    } else if round == 1 {
                        turn
                    } else {
                        turn * 48 + river
                    }
                };
                let p = self.terminal_probs[u - NUM_INTERNAL][bucket];
                let chance = self.compute_chance_prob(round);
                if g.who_folded(u) == self.player as isize {
                    // let ev = eval_fold((-1 * g.get_win_amount(u) as isize + 11) as f64, &p)
                    //     .component_mul(&chance);
                    let ev =
                        eval_fold(-1.0 * g.get_win_amount(u) as f64, &p).component_mul(&chance);
                    return ev;
                } else {
                    let ev = eval_fold(g.get_win_amount(u) as f64, &p).component_mul(&chance);
                    return ev;
                }
            }
        } else if g.get_whose_turn(u) == self.player {
            let mut values = Array2::repeat(-100000000.0);
            for i in 0..TOTAL_ACTIONS {
                if g.can_do_action(i, u) {
                    let v = self.compute_best_response_2(
                        g.do_action(i, u) as usize,
                        g,
                        strat,
                        turn,
                        river,
                        hands,
                    );
                    values.column_mut(i).copy_from(&v);
                }
            }
            let argmax: Vec<_> = values
                .row_iter()
                .map(|row| row.transpose().argmax())
                .collect();
            let indices: Vec<_> = argmax.iter().map(|x| x.0).collect();
            let max_vals = Array1::from_iterator(argmax.iter().map(|x| x.1));
            let mut new_strat = Array2::zeros();
            for (row, col) in indices.iter().enumerate() {
                new_strat[(row, *col)] = 1.0;
            }
            let bucket = {
                if g.get_round(u) == 0 {
                    0
                } else if g.get_round(u) == 1 {
                    turn
                } else {
                    turn * 48 + river
                }
            };
            strat.average_probability[u][bucket] = new_strat;

            return max_vals;
        } else {
            let mut v = Array1::zeros();
            for i in 0..TOTAL_ACTIONS {
                if g.can_do_action(i, u) {
                    v += self.compute_best_response_2(
                        g.do_action(i, u) as usize,
                        g,
                        strat,
                        turn,
                        river,
                        hands,
                    );
                }
            }

            return v;
        }
    }
}
