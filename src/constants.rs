pub const STARTING_STACK: usize = 195;
pub const STARTING_POT: usize = 11;
pub const NO_DONK: bool = true;
pub const AGGRESSOR: usize = 1;
const STATES: (usize, usize) = get_sequences(STARTING_POT, STARTING_STACK);
pub const NUM_INTERNAL: usize = STATES.0;
pub const NUM_TERMINAL: usize = STATES.1;
pub const TOTAL_ACTIONS: usize = 3;
pub const P1_SIZES: [[(usize, usize); 1]; 3] = [[(1, 2)], [(1, 2)], [(1, 2)]];
pub const P2_SIZES: [[(usize, usize); 1]; 3] = [[(1, 2)], [(1, 2)], [(1, 2)]];
pub const NUM_CARDS: u8 = 52;

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
        NO_DONK,
        Some(AGGRESSOR),
        0,
    );
    return (internal, terminal);
}

const fn count_sequences<const P1: usize, const P2: usize>(
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
            while i < P1 && !halted {
                let size = p1_sizes[round][i];
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
                        no_donk,
                        Some(player),
                        num_bets + 1,
                    );
                    i += 1;
                } else {
                    break;
                }
            }
            if i < P1 && !halted {
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
                    no_donk,
                    Some(player),
                    num_bets + 1,
                );
            }
        } else {
            let mut i = 0;
            while i < P2 {
                let size = p2_sizes[round][i];
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
                        no_donk,
                        Some(player),
                        num_bets + 1,
                    );
                    i += 1;
                } else {
                    break;
                }
            }
            if i < P2 {
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
            no_donk, aggressor, num_bets,
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
