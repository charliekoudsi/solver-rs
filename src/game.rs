use hand_eval::{Card, Hand, Rank, Rankable};
use std::cmp::min;

pub const NUM_INTERNAL: usize = 226;
pub const NUM_TERMINAL: usize = 257;
const STARTING_STACK: usize = 80;

// pub struct PlayerView {
//     hole: [Card; 2],
//     board: Vec<Card>,
// }

// impl PlayerView {
//     pub fn new(hole1: Card, board: Option<usize>) -> PlayerView {
//         if let Some(board) = board {
//             return PlayerView {
//                 hole,
//                 board: board as i32,
//             };
//         }
//         return PlayerView { hole, Vec::new() };
//     }

//     #[inline(always)]
//     pub fn get_hand(&self) -> usize {
//         if self.board == -1 {
//             return self.hole;
//         }
//         return 3 * self.hole + self.board as usize;
//     }
// }

// pub struct Abstraction;

// impl Abstraction {
//     const PREFLOP: [usize; 3] = [0, 1, 2];
//     const FLOP: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];

//     #[inline(always)]
//     pub fn abstract_view(&self, view: &PlayerView) -> usize {
//         if view.board == -1 {
//             return Abstraction::PREFLOP[view.get_hand()];
//         }

//         return Abstraction::FLOP[view.get_hand()];
//     }

//     pub fn can_extend(&self, pre: usize, post: usize) -> bool {
//         for i in 0..3 {
//             let pf_view = PlayerView::new(i, None);
//             if self.abstract_view(&pf_view) == pre {
//                 for j in 0..3 {
//                     let f_view = PlayerView::new(i, Some(j));
//                     if self.abstract_view(&f_view) == post {
//                         return true;
//                     }
//                 }
//             }
//         }
//         return false;
//     }
// }

pub struct Game {
    pub rounds: [usize; NUM_INTERNAL],
    whose_turn: [usize; NUM_INTERNAL],
    winner: [isize; NUM_TERMINAL],
    win_amount: [usize; NUM_TERMINAL],
    num_actions: [usize; NUM_INTERNAL],
    pub transition: [[isize; 3]; NUM_INTERNAL],
    parent: [usize; NUM_INTERNAL + NUM_TERMINAL],
}

impl Game {
    pub fn new() -> Game {
        let mut internal: usize = 0;
        let mut terminal: usize = 0;
        let mut rounds: [usize; NUM_INTERNAL] = [0; NUM_INTERNAL];
        let mut whose_turn: [usize; NUM_INTERNAL] = [0; NUM_INTERNAL];
        let mut winner: [isize; NUM_TERMINAL] = [0; NUM_TERMINAL];
        let mut win_amount: [usize; NUM_TERMINAL] = [0; NUM_TERMINAL];
        let mut num_actions: [usize; NUM_INTERNAL] = [0; NUM_INTERNAL];
        let mut transition: [[isize; 3]; NUM_INTERNAL] = [[0; 3]; NUM_INTERNAL];
        let mut parent: [usize; NUM_INTERNAL + NUM_TERMINAL] = [0; NUM_TERMINAL + NUM_INTERNAL];
        construct_sequences(
            0,
            0,
            0,
            true,
            10,
            75,
            &mut rounds,
            &mut whose_turn,
            &mut winner,
            &mut win_amount,
            &mut num_actions,
            &mut transition,
            &mut parent,
            &mut internal,
            &mut terminal,
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

fn construct_sequences(
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
    transition: &mut [[isize; 3]],
    parent: &mut [usize],
    internal: &mut usize,
    terminal: &mut usize,
) -> usize {
    let u = *internal;
    *internal += 1;
    rounds[u] = round;
    whose_turn[u] = player;
    num_actions[u] = 1;

    let opponent = 1 - player;
    if stack > 0 {
        num_actions[u] += 1;
        let raise_size = min((pot + raise) / 2, stack);
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
        );

        transition[u][0] = v as isize;
        parent[v] = u;
    } else {
        transition[u][0] = -1;
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
        );
        transition[u][1] = v as isize;
        parent[v] = u;
    } else {
        if round == 2 {
            let v = *terminal;
            *terminal += 1;
            winner[v] = -1;
            win_amount[v] = pot + raise + stack - STARTING_STACK;
            transition[u][1] = (v as isize) + (NUM_INTERNAL as isize);
            parent[v + NUM_INTERNAL] = u;
        } else {
            if stack > 0 || raise != 0 {
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
                );
                transition[u][1] = v as isize;
                parent[v] = u;
            } else {
                let v = *terminal;
                *terminal += 1;
                winner[v] = -1;
                win_amount[v] = pot + raise + stack - STARTING_STACK;
                transition[u][1] = (v as isize) + (NUM_INTERNAL as isize);
            }
        }
    }

    if raise != 0 {
        num_actions[u] += 1;
        let v = *terminal;
        *terminal += 1;
        winner[v] = opponent as isize;
        win_amount[v] = pot + stack - STARTING_STACK;
        transition[u][2] = (v + NUM_INTERNAL) as isize;
        parent[v + NUM_INTERNAL] = u;
    } else {
        transition[u][2] = -1;
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
