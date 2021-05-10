use nalgebra::{SMatrix, SVector};

pub type Array1 = SVector<f64, COMBOS>;
pub type Array2 = SMatrix<f64, COMBOS, TOTAL_ACTIONS>;
pub type IntArray = SVector<usize, COMBOS>;

pub const COMBOS: usize = 56;
pub const STARTING_STACK: usize = 195;
pub const STARTING_POT: usize = 11;
pub const NO_DONK: bool = true;
pub const AGGRESSOR: usize = 1;
const STATES: (usize, usize) = get_sequences(STARTING_POT, STARTING_STACK);
pub const NUM_INTERNAL: usize = STATES.0;
pub const NUM_TERMINAL: usize = STATES.1;
pub const TOTAL_ACTIONS: usize = 4;
pub const P1_SIZES: [[(usize, usize); 2]; 3] =
    [[(1, 3), (2, 3)], [(1, 3), (2, 3)], [(1, 3), (2, 3)]];
pub const P1_RAISES: [(usize, usize); 1] = [(1, 2)];
pub const P2_SIZES: [[(usize, usize); 2]; 3] =
    [[(1, 3), (2, 3)], [(1, 3), (2, 3)], [(1, 3), (2, 3)]];
pub const P2_RAISES: [(usize, usize); 1] = [(1, 2)];
pub const NUM_CARDS: u8 = 52;
pub const TURN_CARDS: [u8; 52] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51,
];
pub const RIVER_CARDS: [u8; 52] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51,
];
pub const HOLES: [(u8, u8); COMBOS] = get_holes();

const fn get_sequences(pot: usize, stack: usize) -> (usize, usize) {
    let mut internal = 0;
    let mut terminal = 0;
    count_sequences(
        0,
        0,
        0,
        true,
        pot,
        stack,
        &mut internal,
        &mut terminal,
        &P1_SIZES,
        &P2_SIZES,
        &P1_RAISES,
        &P2_RAISES,
        NO_DONK,
        Some(AGGRESSOR),
        0,
    );
    return (internal, terminal);
}

const fn count_sequences<
    const P1: usize,
    const P2: usize,
    const P1_RAISES: usize,
    const P2_RAISES: usize,
>(
    player: usize,
    round: usize,
    raise: usize,
    first_action: bool,
    pot: usize,
    stack: usize,
    internal: &mut usize,
    terminal: &mut usize,
    p1_sizes: &[[(usize, usize); P1]; 3],
    p2_sizes: &[[(usize, usize); P2]; 3],
    p1_raises: &[(usize, usize); P1_RAISES],
    p2_raises: &[(usize, usize); P2_RAISES],
    no_donk: bool,
    aggressor: Option<usize>,
    num_bets: usize,
) {
    let opponent = 1 - player;
    *internal += 1;
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
            let cap = {
                if num_bets == 0 {
                    P1
                } else {
                    P1_RAISES
                }
            };
            while i < cap && !halted {
                let size = {
                    if num_bets == 0 {
                        p1_sizes[round][i]
                    } else {
                        p1_raises[i]
                    }
                };
                let raise_size = (pot + raise) * size.0 / size.1;
                if stack > raise_size {
                    count_sequences(
                        opponent,
                        round,
                        raise_size,
                        false,
                        pot + raise + raise_size,
                        stack - raise_size,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        p1_raises,
                        p2_raises,
                        no_donk,
                        Some(player),
                        num_bets + 1,
                    );
                    i += 1;
                } else {
                    break;
                }
            }
            if i < cap && !halted {
                count_sequences(
                    opponent,
                    round,
                    stack,
                    false,
                    pot + raise + stack,
                    0,
                    internal,
                    terminal,
                    p1_sizes,
                    p2_sizes,
                    p1_raises,
                    p2_raises,
                    no_donk,
                    Some(player),
                    num_bets + 1,
                );
            }
        } else {
            let mut i = 0;
            let cap = {
                if num_bets == 0 {
                    P2
                } else {
                    P2_RAISES
                }
            };
            while i < cap {
                let size = {
                    if num_bets == 0 {
                        p2_sizes[round][i]
                    } else {
                        p2_raises[i]
                    }
                };
                let raise_size = (pot + raise) * size.0 / size.1;
                if stack > raise_size {
                    count_sequences(
                        opponent,
                        round,
                        raise_size,
                        false,
                        pot + raise + raise_size,
                        stack - raise_size,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        p1_raises,
                        p2_raises,
                        no_donk,
                        Some(player),
                        num_bets + 1,
                    );
                    i += 1;
                } else {
                    break;
                }
            }
            if i < cap {
                count_sequences(
                    opponent,
                    round,
                    stack,
                    false,
                    pot + raise + stack,
                    0,
                    internal,
                    terminal,
                    p1_sizes,
                    p2_sizes,
                    p1_raises,
                    p2_raises,
                    no_donk,
                    Some(player),
                    num_bets + 1,
                );
            }
        }
    }

    if first_action {
        // if stack > 0 {
        count_sequences(
            opponent, round, raise, false, pot, stack, internal, terminal, p1_sizes, p2_sizes,
            p1_raises, p2_raises, no_donk, aggressor, num_bets,
        );
    // }
    } else {
        if round == 2 {
            *terminal += 1;
        } else {
            if stack > 0 {
                if num_bets == 0 {
                    count_sequences(
                        0,
                        round + 1,
                        0,
                        true,
                        pot + raise,
                        stack,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        p1_raises,
                        p2_raises,
                        no_donk,
                        None,
                        0,
                    );
                } else {
                    count_sequences(
                        0,
                        round + 1,
                        0,
                        true,
                        pot + raise,
                        stack,
                        internal,
                        terminal,
                        p1_sizes,
                        p2_sizes,
                        p1_raises,
                        p2_raises,
                        no_donk,
                        Some(opponent),
                        0,
                    );
                }
            } else {
                *terminal += 1;
            }
        }
    }

    if raise != 0 {
        *terminal += 1;
    }
}

const fn get_holes() -> [(u8, u8); COMBOS] {
    // let mut holes = [(0, 0); COMBOS];

    // let mut i = 0;
    // let mut idx = 0;
    // while i < NUM_CARDS {
    //     let mut j = i + 1;
    //     while j < NUM_CARDS {
    //         holes[idx] = (i, j);
    //         idx += 1;
    //         j += 1;
    //     }
    //     i += 1;
    // }

    // return holes;
    return [
        (48, 49),
        (48, 50),
        (48, 51),
        (49, 50),
        (49, 51),
        (50, 51),
        (44, 48),
        (45, 49),
        (46, 50),
        (47, 51),
        (44, 45),
        (44, 46),
        (44, 47),
        (45, 46),
        (45, 47),
        (46, 47),
        (40, 44),
        (41, 45),
        (42, 46),
        (43, 47),
        (40, 48),
        (41, 49),
        (42, 50),
        (43, 51),
        (40, 41),
        (40, 42),
        (40, 43),
        (41, 42),
        (41, 43),
        (42, 43),
        (36, 40),
        (37, 41),
        (38, 42),
        (39, 43),
        (36, 37),
        (36, 38),
        (36, 39),
        (37, 38),
        (37, 39),
        (38, 39),
        (32, 36),
        (33, 37),
        (34, 38),
        (32, 33),
        (32, 34),
        (33, 34),
        (28, 29),
        (28, 30),
        (29, 30),
        (28, 32),
        (29, 33),
        (30, 34),
        (20, 24),
        (21, 25),
        (22, 26),
        (23, 27),
    ];
}
