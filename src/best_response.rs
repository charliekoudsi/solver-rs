use crate::constants::{NUM_INTERNAL, NUM_TERMINAL, STARTING_POT, TOTAL_ACTIONS};
use crate::game::{get_bucket, get_turn_bucket, Game};
use crate::regret::SafeRegretStrategy as RegretStrategy;
use std::collections::HashMap;

type TerminalProb = Vec<Vec<f32>>;
type Range = Vec<(u8, u8)>;

pub struct BestResponse {
    player: usize,
    pub terminal_probs: TerminalProb,
    range: Range,
    opp_range: Range,
    opp_combos: Vec<u8>,
    flop: [u8; 3],
}

fn compute_terminal_probabilities(
    player: usize,
    flop: &[u8; 3],
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
    range: &Range,
) {
    for i in 0..range.len() {
        compute_flop_probabilities(0, player, flop, i, range, 1.0, strat, prob, g);
    }
}

fn compute_flop_probabilities(
    u: usize,
    player: usize,
    flop: &[u8; 3],
    range_bucket: usize,
    range: &Range,
    reach: f32,
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    if g.is_terminal(u) {
        prob[u - NUM_INTERNAL][range_bucket] = reach;
    } else if g.get_round(u) == 1 {
        let (card1, card2) = range[range_bucket];
        let mut turn = 0;
        for i in 0..36 {
            if card1 == i || card2 == i || flop[0] == i || flop[1] == i || flop[2] == i {
                continue;
            }
            compute_turn_probabilities(
                u,
                player,
                flop,
                turn,
                range_bucket,
                range,
                reach,
                strat,
                prob,
                g,
            );
            turn += 1;
        }
    } else if g.get_whose_turn(u) == player {
        let actions = strat.get_average_normalized_probability(u, range_bucket, g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_flop_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    range_bucket,
                    range,
                    reach * actions[i],
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
                    range_bucket,
                    range,
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
    range_bucket: usize,
    range: &Range,
    reach: f32,
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    if g.is_terminal(u) {
        prob[u - NUM_INTERNAL][range_bucket * 31 + turn as usize] = reach;
    } else if g.get_round(u) == 2 {
        let (card1, card2) = range[range_bucket];
        let mut river = 0;
        for i in 0..36 {
            if card1 == i || card2 == i || flop[0] == i || flop[1] == i || flop[2] == i || turn == i
            {
                continue;
            }
            compute_river_probabilities(
                u,
                player,
                flop,
                turn,
                river,
                range_bucket,
                range,
                reach,
                strat,
                prob,
                g,
            );
            river += 1;
        }
    } else if g.get_whose_turn(u) == player {
        let actions =
            strat.get_average_normalized_probability(u, range_bucket * 31 + turn as usize, g);
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_turn_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    turn,
                    range_bucket,
                    range,
                    reach * actions[i],
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
                    range_bucket,
                    range,
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
    range_bucket: usize,
    range: &Range,
    reach: f32,
    strat: &RegretStrategy,
    prob: &mut TerminalProb,
    g: &Game,
) {
    if g.is_terminal(u) {
        prob[u - NUM_INTERNAL][(range_bucket * 31 + turn as usize) * 30 + river as usize] = reach;
    } else if g.get_whose_turn(u) == player {
        let actions = strat.get_average_normalized_probability(
            u,
            (range_bucket * 31 + turn as usize) * 30 + river as usize,
            g,
        );
        for i in 0..TOTAL_ACTIONS {
            if g.can_do_action(i, u) {
                compute_river_probabilities(
                    g.do_action(i, u) as usize,
                    player,
                    flop,
                    turn,
                    river,
                    range_bucket,
                    range,
                    reach * actions[i],
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
                    range_bucket,
                    range,
                    reach,
                    strat,
                    prob,
                    g,
                )
            }
        }
    }
}

fn compute_opp_combos(range: &Range, opp_range: &Range) -> Vec<u8> {
    let mut combos = vec![0; range.len()];
    for i in 0..range.len() {
        let mut opp_combos = 0;
        let (card1, card2) = range[i];
        for (opp1, opp2) in opp_range.iter() {
            if *opp1 != card1 && *opp1 != card2 && *opp2 != card1 && *opp2 != card2 {
                opp_combos += 1;
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
        let mut probability: TerminalProb = vec![vec![0.0; 1]; NUM_TERMINAL];
        for i in 0..NUM_TERMINAL {
            if g.get_round(i + NUM_INTERNAL) == 0 {
                probability[i] = vec![0.0; range.len()];
            } else if g.get_round(i + NUM_INTERNAL) == 1 {
                probability[i] = vec![0.0; range.len() * 31];
            } else {
                probability[i] = vec![0.0; range.len() * 31 * 30];
            }
        }
        compute_terminal_probabilities(1 - p, &flop, strat, &mut probability, g, &range);
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

    fn compute_chance_prob(&self, round: usize, hand_bucket: usize) -> f32 {
        let combos = self.range.len() as f32;
        let opp_combos = self.opp_combos[hand_bucket] as f32;
        if round == 0 {
            return 1.0 / (combos * opp_combos);
        } else if round == 1 {
            return 1.0 / (combos * opp_combos * 29.0);
        }
        return 1.0 / (combos * opp_combos * 29.0 * 28.0);
    }

    pub fn compute_best_response(
        &self,
        u: usize,
        g: &Game,
        strat: &mut RegretStrategy,
        hand: Option<usize>,
        turn: Option<(usize, usize)>,
        river: Option<(usize, usize)>,
        map: &HashMap<((u8, u8), (u8, u8), u8, u8), i8>,
    ) -> f32 {
        if g.is_terminal(u) {
            if g.is_showdown(u) {
                let mut ev = 0.0;
                if let Some((_, r)) = river {
                    let h_bucket = hand.expect("Reached showdown, but no hand provided");
                    let (_, t) = turn.expect("Reached showdown, but no turn provided");
                    let t = t as u8;
                    let r = r as u8;
                    let h = self.range[h_bucket];
                    let chance = self.compute_chance_prob(2, h_bucket);
                    for i in 0..self.opp_range.len() {
                        let (card1, card2) = self.opp_range[i];
                        if card1 != t
                            && card2 != t
                            && card1 != r
                            && card2 != r
                            && card1 != h.0
                            && card1 != h.1
                            && card2 != h.0
                            && card2 != h.1
                        {
                            let (t_b, r_b) = get_bucket(card1, card2, &self.flop, t, r);
                            let p = self.terminal_probs[u - NUM_INTERNAL]
                                [(i * 31 + t_b) * 30 + r_b as usize];
                            let winner = *map.get(&(h, (card1, card2), t, r)).expect("not set");
                            let amount = g.get_win_amount(u);
                            if winner == 1 {
                                ev += amount as f32 * p * chance
                            } else if winner == -1 {
                                ev += (-1 * (amount - STARTING_POT) as isize) as f32 * p * chance;
                            } else {
                                ev += STARTING_POT as f32 / 2.0 * p * chance;
                            }
                        }
                    }
                } else if let Some((_, t)) = turn {
                    let h_bucket = hand.expect("Reached showdown, but no hand provided");
                    let h = self.range[h_bucket];
                    let t = t as u8;
                    let chance = self.compute_chance_prob(2, h_bucket);
                    for j in 0..36 {
                        if j == self.flop[0]
                            || j == self.flop[1]
                            || j == self.flop[2]
                            || j == t
                            || j == h.0
                            || j == h.1
                        {
                            continue;
                        }
                        for i in 0..self.opp_range.len() {
                            let (card1, card2) = self.opp_range[i];
                            if card1 != t
                                && card2 != t
                                && card1 != j
                                && card2 != j
                                && card1 != h.0
                                && card1 != h.1
                                && card2 != h.0
                                && card2 != h.1
                            {
                                let t_b = get_turn_bucket(card1, card2, &self.flop, t);
                                let p = self.terminal_probs[u - NUM_INTERNAL][i * 31 + t_b];
                                let winner = *map.get(&(h, (card1, card2), t, j)).expect("not set");
                                let amount = g.get_win_amount(u);
                                if winner == 1 {
                                    ev += amount as f32 * p * chance
                                } else if winner == -1 {
                                    ev +=
                                        (-1 * (amount - STARTING_POT) as isize) as f32 * p * chance;
                                } else {
                                    ev += STARTING_POT as f32 / 2.0 * p * chance;
                                }
                            }
                        }
                    }
                } else {
                    let h_bucket = hand.expect("Reached showdown, but no hand provided");
                    let h = self.range[h_bucket];
                    let chance = self.compute_chance_prob(2, h_bucket);
                    let mut t = 0;
                    for k in 0..36 {
                        if k == self.flop[0]
                            || k == self.flop[1]
                            || k == self.flop[2]
                            || k == h.0
                            || k == h.1
                        {
                            continue;
                        }
                        let mut r = 0;
                        for j in 0..36 {
                            if j == self.flop[0]
                                || j == self.flop[1]
                                || j == self.flop[2]
                                || j == k
                                || j == h.0
                                || j == h.1
                            {
                                continue;
                            }
                            for i in 0..self.opp_range.len() {
                                let (card1, card2) = self.opp_range[i];
                                if card1 != j
                                    && card2 != j
                                    && card1 != k
                                    && card2 != k
                                    && card1 != h.0
                                    && card1 != h.1
                                    && card2 != h.0
                                    && card2 != h.1
                                {
                                    let p = self.terminal_probs[u - NUM_INTERNAL][i];
                                    let winner =
                                        *map.get(&(h, (card1, card2), k, j)).expect("not set");
                                    let amount = g.get_win_amount(u);
                                    if winner == 1 {
                                        ev += amount as f32 * p * chance
                                    } else if winner == -1 {
                                        ev += (-1 * (amount - STARTING_POT) as isize) as f32
                                            * p
                                            * chance;
                                    } else {
                                        ev += STARTING_POT as f32 / 2.0 * p * chance;
                                    }
                                }
                            }
                            r += 1;
                        }
                        t += 1;
                    }
                }
                return ev;
            } else {
                let result: isize = {
                    if g.who_folded(u) == self.player as isize {
                        -1
                    } else {
                        1
                    }
                };
                let mut p = 0.0;
                if g.get_round(u) == 0 {
                    let h_bucket = hand.expect("Reached fold, but no hand provided");
                    let h = self.range[h_bucket];
                    let chance = self.compute_chance_prob(0, h_bucket);
                    for i in 0..self.opp_range.len() {
                        let (card1, card2) = self.opp_range[i];
                        if card1 != h.0 && card1 != h.1 && card2 != h.0 && card2 != h.1 {
                            p += self.terminal_probs[u - NUM_INTERNAL][i] * chance;
                        }
                    }
                } else if g.get_round(u) == 1 {
                    let h_bucket = hand.expect("Reached fold, but no hand provided");
                    let (_, t) = turn.expect("Reached fold, but no turn provided");
                    let t = t as u8;
                    let h = self.range[h_bucket];
                    let chance = self.compute_chance_prob(1, h_bucket);
                    for i in 0..self.opp_range.len() {
                        let (card1, card2) = self.opp_range[i];
                        if card1 != h.0
                            && card1 != h.1
                            && card2 != h.0
                            && card2 != h.1
                            && card1 != t
                            && card2 != t
                        {
                            let t_b = get_turn_bucket(card1, card2, &self.flop, t);
                            p += self.terminal_probs[u - NUM_INTERNAL][i * 31 + t_b] * chance;
                        }
                    }
                } else {
                    let h_bucket = hand.expect("Reached fold, but no hand provided");
                    let (_, t) = turn.expect("Reached fold, but no turn provided");
                    let t = t as u8;
                    let (_, r) = river.expect("Reached fold, but no river provided");
                    let r = r as u8;
                    let h = self.range[h_bucket];
                    let chance = self.compute_chance_prob(2, h_bucket);
                    for i in 0..self.opp_range.len() {
                        let (card1, card2) = self.opp_range[i];
                        if card1 != h.0
                            && card1 != h.1
                            && card2 != h.0
                            && card2 != h.1
                            && card1 != t
                            && card2 != t
                            && card1 != r
                            && card2 != r
                        {
                            let (t_b, r_b) = get_bucket(card1, card2, &self.flop, t, r);
                            p += self.terminal_probs[u - NUM_INTERNAL][(i * 31 + t_b) * 30 + r_b]
                                * chance;
                        }
                    }
                }
                if result == 1 {
                    return p * g.get_win_amount(u) as f32;
                } else {
                    return p * (-1 * (g.get_win_amount(u) - STARTING_POT) as isize) as f32;
                }
            }
        } else if hand == None {
            let mut v = 0.0;
            for i in 0..self.range.len() {
                v += self.compute_best_response(u, g, strat, Some(i), None, None, map);
            }
            return v;
        } else if g.get_round(u) == 1 && turn == None {
            let mut v = 0.0;
            let mut t = 0;
            let h = hand.expect("Reached turn, but no hand provided");
            let h = self.range[h];
            for i in 0..36 {
                if i == self.flop[0]
                    || i == self.flop[1]
                    || i == self.flop[2]
                    || i == h.0
                    || i == h.1
                {
                    continue;
                }
                v +=
                    self.compute_best_response(u, g, strat, hand, Some((t, i as usize)), None, map);
                t += 1;
            }
            return v;
        } else if g.get_round(u) == 2 && river == None {
            let mut v = 0.0;
            let mut r = 0;
            let h = hand.expect("Reached river, but no hand provided");
            let (_, t) = turn.expect("Reached river, but no turn provided");
            let t = t as u8;
            let h = self.range[h];
            for i in 0..36 {
                if i == self.flop[0]
                    || i == self.flop[1]
                    || i == self.flop[2]
                    || i == h.0
                    || i == h.1
                    || i == t as u8
                {
                    continue;
                }
                v +=
                    self.compute_best_response(u, g, strat, hand, turn, Some((r, i as usize)), map);
                r += 1;
            }
            return v;
        } else if g.get_whose_turn(u) == self.player {
            let mut max_val = -100000000.0;
            let mut max_index: usize = 0;
            for i in 0..TOTAL_ACTIONS {
                if g.can_do_action(i, u) {
                    let v = self.compute_best_response(
                        g.do_action(i, u) as usize,
                        g,
                        strat,
                        hand,
                        turn,
                        river,
                        map,
                    );
                    if v > max_val {
                        max_val = v;
                        max_index = i;
                    }
                }
            }
            let mut tuple = [0.0; TOTAL_ACTIONS];
            tuple[max_index] = 1.0;
            if let Some(r) = river {
                let h = hand.expect("Reached river, but no hand provided");
                let t = turn.expect("Reached river, but no turn provided");
                strat.average_probability[u][(h * 31 + t.0) * 30 + r.0] = tuple;
            } else if let Some(t) = turn {
                let h = hand.expect("Reached turn, but no hand provided");
                strat.average_probability[u][h * 31 + t.0] = tuple;
            } else {
                let h = hand.expect("Reached flop, but no hand provided");
                strat.average_probability[u][h] = tuple;
            }
            return max_val;
        } else {
            let mut v = 0.0;
            for i in 0..TOTAL_ACTIONS {
                if g.can_do_action(i, u) {
                    v += self.compute_best_response(
                        g.do_action(i, u) as usize,
                        g,
                        strat,
                        hand,
                        turn,
                        river,
                        map,
                    );
                }
            }
            return v;
        }
    }
}
