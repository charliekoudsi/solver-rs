use crate::constants::*;
use hand_eval::{Hand, Rankable};

pub struct Game {
    pub rounds: [usize; NUM_INTERNAL],
    whose_turn: [usize; NUM_INTERNAL],
    winner: [isize; NUM_TERMINAL],
    win_amount: [usize; NUM_TERMINAL],
    num_actions: [usize; NUM_INTERNAL],
    pub transition: [[isize; TOTAL_ACTIONS]; NUM_INTERNAL],
    pub parent: [usize; NUM_INTERNAL + NUM_TERMINAL],
}

impl Game {
    pub fn new() -> Game {
        let mut internal: usize = 0;
        let mut terminal: usize = 0;
        println!("{} {}", NUM_INTERNAL, NUM_TERMINAL);
        let mut rounds: [usize; NUM_INTERNAL] = [0; NUM_INTERNAL];
        let mut whose_turn: [usize; NUM_INTERNAL] = [0; NUM_INTERNAL];
        let mut winner: [isize; NUM_TERMINAL] = [0; NUM_TERMINAL];
        let mut win_amount: [usize; NUM_TERMINAL] = [0; NUM_TERMINAL];
        let mut num_actions: [usize; NUM_INTERNAL] = [0; NUM_INTERNAL];
        let mut transition: [[isize; TOTAL_ACTIONS]; NUM_INTERNAL] =
            [[-1; TOTAL_ACTIONS]; NUM_INTERNAL];
        let mut parent: [usize; NUM_INTERNAL + NUM_TERMINAL] = [0; NUM_TERMINAL + NUM_INTERNAL];
        construct_sequences(
            0,
            0,
            0,
            true,
            STARTING_POT,
            STARTING_STACK,
            &mut rounds,
            &mut whose_turn,
            &mut winner,
            &mut win_amount,
            &mut num_actions,
            &mut transition,
            &mut parent,
            &mut internal,
            &mut terminal,
            &P1_SIZES,
            &P2_SIZES,
            NO_DONK,
            Some(AGGRESSOR),
            0,
        );
        return Game {
            rounds,
            whose_turn,
            winner,
            win_amount,
            num_actions,
            transition,
            parent,
        };
    }

    #[inline(always)]
    pub fn is_terminal(&self, u: usize) -> bool {
        return u >= NUM_INTERNAL;
    }

    #[inline(always)]
    pub fn get_round(&self, u: usize) -> usize {
        if self.is_terminal(u) {
            return self.rounds[self.parent[u]];
        }
        return self.rounds[u];
    }

    #[inline(always)]
    pub fn get_whose_turn(&self, u: usize) -> usize {
        return self.whose_turn[u];
    }

    #[inline(always)]
    pub fn is_fold(&self, u: usize) -> bool {
        return self.is_terminal(u) && self.winner[u - NUM_INTERNAL] != -1;
    }

    #[inline(always)]
    pub fn is_showdown(&self, u: usize) -> bool {
        return self.is_terminal(u) && self.winner[u - NUM_INTERNAL] == -1;
    }

    #[inline(always)]
    pub fn winner_at_fold(&self, u: usize) -> isize {
        return self.winner[u - NUM_INTERNAL];
    }

    #[inline(always)]
    pub fn get_win_amount(&self, u: usize) -> usize {
        return self.win_amount[u - NUM_INTERNAL];
    }

    #[inline(always)]
    pub fn who_folded(&self, u: usize) -> isize {
        return 1 - self.winner_at_fold(u);
    }

    #[inline(always)]
    pub fn get_num_actions(&self, u: usize) -> usize {
        return self.num_actions[u];
    }

    #[inline(always)]
    pub fn can_do_action(&self, action: usize, u: usize) -> bool {
        return self.transition[u][action] != -1;
    }

    #[inline(always)]
    pub fn do_action(&self, action: usize, u: usize) -> isize {
        return self.transition[u][action];
    }
}

fn construct_sequences<const P1: usize, const P2: usize>(
    player: usize,
    round: usize,
    raise: usize,
    first_action: bool,
    pot: usize,
    stack: usize,
    rounds: &mut [usize],
    whose_turn: &mut [usize],
    winner: &mut [isize],
    win_amount: &mut [usize],
    num_actions: &mut [usize],
    transition: &mut [[isize; TOTAL_ACTIONS]],
    parent: &mut [usize],
    internal: &mut usize,
    terminal: &mut usize,
    p1_sizes: &[[(usize, usize); P1]; 3],
    p2_sizes: &[[(usize, usize); P2]; 3],
    no_donk: bool,
    aggressor: Option<usize>,
    num_bets: usize,
) -> usize {
    let u = *internal;
    *internal += 1;
    rounds[u] = round;
    whose_turn[u] = player;
    num_actions[u] = 1;

    let opponent = 1 - player;
    if stack > 0 {
        if player == 0 {
            let halted: bool = {
                if no_donk {
                    if let Some(a) = aggressor {
                        if a == 1 && first_action {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            let mut i = 0;
            while i < P1 && !halted {
                let size = p1_sizes[round][i];
                let raise_size = (pot + raise) * size.0 / size.1;
                if stack > raise_size {
                    num_actions[u] += 1;
                    let v = construct_sequences(
                        opponent,
                        round,
                        raise_size,
                        false,
                        pot + raise + raise_size,
                        stack - raise_size,
                        rounds,
                        whose_turn,
                        winner,
                        win_amount,
                        num_actions,
                        transition,
                        parent,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        no_donk,
                        Some(player),
                        num_bets + 1,
                    );
                    transition[u][i] = v as isize;
                    parent[v] = u;
                    i += 1;
                } else {
                    break;
                }
            }
            if i < P1 && !halted {
                num_actions[u] += 1;
                let v = construct_sequences(
                    opponent,
                    round,
                    stack,
                    false,
                    pot + raise + stack,
                    0,
                    rounds,
                    whose_turn,
                    winner,
                    win_amount,
                    num_actions,
                    transition,
                    parent,
                    internal,
                    terminal,
                    p1_sizes,
                    p2_sizes,
                    no_donk,
                    Some(player),
                    num_bets + 1,
                );
                transition[u][i] = v as isize;
                parent[v] = u;
            }
        } else {
            let mut i = 0;
            while i < P2 {
                let size = p2_sizes[round][i];
                let raise_size = (pot + raise) * size.0 / size.1;
                if stack > raise_size {
                    num_actions[u] += 1;
                    let v = construct_sequences(
                        opponent,
                        round,
                        raise_size,
                        false,
                        pot + raise + raise_size,
                        stack - raise_size,
                        rounds,
                        whose_turn,
                        winner,
                        win_amount,
                        num_actions,
                        transition,
                        parent,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        no_donk,
                        Some(player),
                        num_bets + 1,
                    );
                    transition[u][i] = v as isize;
                    parent[v] = u;
                    i += 1;
                } else {
                    break;
                }
            }
            if i < P2 {
                num_actions[u] += 1;
                let v = construct_sequences(
                    opponent,
                    round,
                    stack,
                    false,
                    pot + raise + stack,
                    0,
                    rounds,
                    whose_turn,
                    winner,
                    win_amount,
                    num_actions,
                    transition,
                    parent,
                    internal,
                    terminal,
                    p1_sizes,
                    p2_sizes,
                    no_donk,
                    Some(player),
                    num_bets + 1,
                );
                transition[u][i] = v as isize;
                parent[v] = u;
            }
        }
    }

    if first_action {
        let v = construct_sequences(
            opponent,
            round,
            0,
            false,
            pot,
            stack,
            rounds,
            whose_turn,
            winner,
            win_amount,
            num_actions,
            transition,
            parent,
            internal,
            terminal,
            p1_sizes,
            p2_sizes,
            no_donk,
            aggressor,
            num_bets,
        );
        transition[u][TOTAL_ACTIONS - 2] = v as isize;
        parent[v] = u;
    } else {
        if round == 2 {
            let v = *terminal;
            *terminal += 1;
            winner[v] = -1;
            win_amount[v] = pot + raise + stack - STARTING_STACK;
            transition[u][TOTAL_ACTIONS - 2] = (v as isize) + (NUM_INTERNAL as isize);
            parent[v + NUM_INTERNAL] = u;
        } else {
            if stack > 0 {
                if num_bets == 0 {
                    let v = construct_sequences(
                        0,
                        round + 1,
                        0,
                        true,
                        pot + raise,
                        stack,
                        rounds,
                        whose_turn,
                        winner,
                        win_amount,
                        num_actions,
                        transition,
                        parent,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        no_donk,
                        None,
                        0,
                    );
                    transition[u][TOTAL_ACTIONS - 2] = v as isize;
                    parent[v] = u;
                } else {
                    let v = construct_sequences(
                        0,
                        round + 1,
                        0,
                        true,
                        pot + raise,
                        stack,
                        rounds,
                        whose_turn,
                        winner,
                        win_amount,
                        num_actions,
                        transition,
                        parent,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        no_donk,
                        Some(opponent),
                        0,
                    );
                    transition[u][TOTAL_ACTIONS - 2] = v as isize;
                    parent[v] = u;
                }
            } else {
                let v = *terminal;
                *terminal += 1;
                winner[v] = -1;
                win_amount[v] = pot + raise + stack - STARTING_STACK;
                transition[u][TOTAL_ACTIONS - 2] = (v as isize) + (NUM_INTERNAL as isize);
                parent[v + NUM_INTERNAL] = u;
            }
        }
    }

    if raise != 0 {
        num_actions[u] += 1;
        let v = *terminal;
        *terminal += 1;
        winner[v] = opponent as isize;
        win_amount[v] = pot + stack - STARTING_STACK;
        transition[u][TOTAL_ACTIONS - 1] = (v + NUM_INTERNAL) as isize;
        parent[v + NUM_INTERNAL] = u;
    } else {
        transition[u][TOTAL_ACTIONS - 1] = -1;
    }

    return u;
}

#[inline(always)]
pub fn evaluate_winner(p1: (u8, u8), p2: (u8, u8), board: &[u8; 5]) -> i8 {
    let p1_hand = Hand::new_with_u8(p1.0, p1.1, board);
    let p2_hand = Hand::new_with_u8(p2.0, p2.1, board);
    if p1_hand.rank() > p2_hand.rank() {
        return 1;
    }
    if p1_hand.rank() < p2_hand.rank() {
        return -1;
    }
    return 0;
}

#[inline]
fn get_river_bucket(one: u8, two: u8, flop: &[u8; 3], turn: u8, river: u8) -> usize {
    let mut baseline = {
        if river > flop[2] {
            river - 3
        } else if river > flop[1] {
            river - 2
        } else if river > flop[0] {
            river - 1
        } else {
            river
        }
    };
    if river > turn {
        baseline -= 1;
    }
    if river > two {
        (baseline - 2) as usize
    } else if river > one {
        (baseline - 1) as usize
    } else {
        baseline as usize
    }
}

// #[inline]
// fn get_river_buckets(
//     p1_one: u8,
//     p1_two: u8,
//     p2_one: u8,
//     p2_two: u8,
//     flop: &[u8; 3],
//     turn: u8,
//     river: u8,
// ) -> (usize, usize) {
//     let p1 = get_river_bucket(p1_one, p1_two, flop, turn, river);
//     let p2 = get_river_bucket(p2_one, p2_two, flop, turn, river);
//     return (p1, p2);
// }

#[inline]
pub fn get_turn_bucket(one: u8, two: u8, flop: &[u8; 3], turn: u8) -> usize {
    let baseline = {
        if turn > flop[2] {
            turn - 3
        } else if turn > flop[1] {
            turn - 2
        } else if turn > flop[0] {
            turn - 1
        } else {
            turn
        }
    };
    if turn > two {
        (baseline - 2) as usize
    } else if turn > one {
        (baseline - 1) as usize
    } else {
        baseline as usize
    }
}

// #[inline]
// fn get_turn_buckets(
//     p1_one: u8,
//     p1_two: u8,
//     p2_one: u8,
//     p2_two: u8,
//     flop: &[u8; 3],
//     turn: u8,
// ) -> (usize, usize) {
//     let p1 = get_turn_bucket(p1_one, p1_two, flop, turn);
//     let p2 = get_turn_bucket(p2_one, p2_two, flop, turn);
//     return (p1, p2);
// }

#[inline]
pub fn get_bucket(one: u8, two: u8, flop: &[u8; 3], turn: u8, river: u8) -> (usize, usize) {
    let turn_b = get_turn_bucket(one, two, flop, turn);
    let river_b = get_river_bucket(one, two, flop, turn, river);
    return (turn_b, river_b);
}

#[inline]
pub fn get_buckets(
    p1_one: u8,
    p1_two: u8,
    p2_one: u8,
    p2_two: u8,
    flop: &[u8; 3],
    turn: u8,
    river: u8,
) -> ((usize, usize), (usize, usize)) {
    let p1 = get_bucket(p1_one, p1_two, flop, turn, river);
    let p2 = get_bucket(p2_one, p2_two, flop, turn, river);
    return ((p1.0, p2.0), (p1.1, p2.1));
}
