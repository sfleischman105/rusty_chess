use super::threads::Thread;
use super::UCILimit;

use rand::{Rng};

//use test::{self,Bencher};

use std::cmp::{min,max};
use std::sync::atomic::Ordering;

use pleco::{MoveList,Board,BitMove};
use pleco::core::*;
use pleco::board::eval::*;
use pleco::tools::tt::*;


use super::misc::*;
use super::MAX_PLY;
use super::TT_TABLE;

const THREAD_DIST: usize = 20;
//                                      1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
static SKIP_SIZE: [u16; THREAD_DIST] = [1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4];
static START_PLY: [u16; THREAD_DIST] = [0, 1, 0, 1, 2, 3, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7];


pub struct ThreadSearcher<'a> {
    pub thread: &'a mut Thread,
    pub limit: UCILimit,
    pub board: Board
}

impl<'a> ThreadSearcher<'a> {
    pub fn search_root(&mut self) {
        assert!(self.board.depth() == 0);
        if self.stop() {
            return;
        }
        if self.use_stdout() {
            println!("info id {} start", self.thread.id);
        }

        let max_depth = if self.limit.is_depth() {
            self.limit.depth_limit()
        } else {
            MAX_PLY
        };

        let start_ply: u16 = START_PLY[self.thread.id % THREAD_DIST];
        let skip_size: u16 = SKIP_SIZE[self.thread.id % THREAD_DIST];
        let mut depth: u16 = start_ply;

        let mut delta: i32 = NEG_INFINITY as i32;
        let mut best_value: i32 = NEG_INFINITY as i32;
        let mut alpha: i32 = NEG_INFINITY as i32;
        let mut beta: i32 = INFINITY as i32;

        self.thread.root_moves.shuffle(self.thread.id, &self.board);


        while !self.stop() && depth < max_depth {
            self.thread.root_moves.rollback();

            if depth >= 5 {
                delta = 18;
                alpha = max(self.thread.root_moves.prev_best_score() - delta, NEG_INFINITY as i32);
                beta = min(self.thread.root_moves.prev_best_score() + delta, INFINITY as i32);
            }

            'aspiration_window: loop {

                best_value = self.search::<PV>(alpha, beta, depth) as i32;
                self.thread.root_moves.sort();

                if self.stop() {
                    break 'aspiration_window;
                }

                if best_value <= alpha {
                    alpha = max(best_value - delta, NEG_INFINITY as i32);
                } else if best_value >= beta {
                    beta = min(best_value + delta, INFINITY as i32);
                } else {
                    break 'aspiration_window;
                }
                delta += (delta / 4) + 5;

                assert!(alpha >= NEG_INFINITY as i32);
                assert!(beta <= INFINITY as i32);
            }

            self.thread.root_moves.sort();
            if self.use_stdout() {
                println!("info id {} depth {} stop {}",self.thread.id, depth, self.stop());
            }
            if !self.stop() {
                self.thread.depth_completed.store(depth,Ordering::Relaxed);
            }
            depth += skip_size;
        }
//        self.print_all_moves();
    }

    fn search<N: PVNode>(&mut self, mut alpha: i32, beta: i32, max_depth: u16) -> i32 {

        let is_pv: bool = N::is_pv();
        let old_alpha = alpha;
        let at_root: bool = self.board.depth() == 0;
        let zob = self.board.zobrist();
        let (tt_hit, tt_entry): (bool, &mut Entry) = TT_TABLE.probe(zob);
        let tt_value = if tt_hit {tt_entry.score as i32} else {0};
        let in_check: bool = self.board.in_check();
        let ply = self.board.depth();

        let mut best_move = BitMove::null();

        let mut value = NEG_INFINITY as i32;
        let mut best_value = NEG_INFINITY as i32;
        let mut moves_played = 0;

        let mut pos_eval: i32 = 0;

        if ply >= max_depth || self.stop() {
            return Eval::eval_low(&self.board) as i32;
        }

        let plys_to_zero = max_depth - ply;

        if !at_root {
            if alpha >= beta {
                return alpha
            }
        }

        if !is_pv
            && tt_hit
            && tt_entry.depth as u16 >= plys_to_zero
            && tt_value != 0
            && correct_bound_eq(tt_value, beta, tt_entry.node_type()) {
            return tt_value;
        }

        if in_check {
            pos_eval = 0;
        } else {
            if tt_hit {
                if tt_entry.eval == 0 {
                    pos_eval = Eval::eval_low(&self.board) as i32;
                }
                if tt_value != 0 && correct_bound(tt_value, pos_eval, tt_entry.node_type()) {
                    pos_eval = tt_value;
                }
            } else {
                pos_eval = Eval::eval_low(&self.board) as i32;
                tt_entry.place(zob, BitMove::null(), 0, pos_eval as i16, 0, NodeBound::NoBound);
            }
        }

        if !in_check {

            if ply > 3
                && ply < 7
                && pos_eval - futility_margin(ply) >= beta && pos_eval < 10000 {
                return pos_eval;
            }
        }

        #[allow(unused_mut)]
        let mut moves: MoveList = if at_root {
            self.thread.root_moves.to_list()
        } else {
            self.board.generate_pseudolegal_moves()
        };

        if moves.is_empty() {
            if self.board.in_check() {
                return MATE as i32 + (ply as i32);
            } else {
                return STALEMATE as i32;
            }
        }

        if !at_root {
            mvv_lva_sort(&mut moves, &self.board);
        }


        for (i, mov) in moves.iter().enumerate() {
            if at_root || self.board.legal_move(*mov) {
                moves_played += 1;
                let gives_check: bool = self.board.gives_check(*mov);
                self.board.apply_unknown_move(*mov, gives_check);
                let do_full_depth: bool = if max_depth >= 3 && moves_played > 1 && ply >= 2 {
                    if in_check || gives_check {
                        value = -self.search::<NonPV>(-(alpha+1), -alpha, max_depth - 1);
                    } else {
                        value = -self.search::<NonPV>(-(alpha+1), -alpha, max_depth - 2);
                    }
                    value > alpha
                } else {
                    !is_pv || moves_played > 1
                };
                if do_full_depth {
                    value = -self.search::<NonPV>(-(alpha+1), -alpha, max_depth);
                }
                if is_pv && (moves_played == 1 || (value > alpha && (at_root || value < beta))) {
                    value = -self.search::<PV>(-beta, -alpha, max_depth);
                }
                self.board.undo_move();
                assert!(value > NEG_INFINITY as i32);
                assert!(value < INFINITY as i32);
                if self.stop() {
                    return 0;
                }
                if at_root {
                    if (moves_played == 1 || value as i32 > alpha) {
                        self.thread.root_moves.insert_score_depth(i,value, max_depth);
                    } else {
                        self.thread.root_moves.insert_score(i, NEG_INFINITY as i32);
                    }
                }

                if value > best_value {
                    best_value = value;

                    if value > alpha {
                        best_move = *mov;
                        if is_pv && value < beta {
                            alpha = value;
                        } else {
//                            assert!(value >= beta);
                           break;
                        }
                    }
                }
            }
        }

        if moves_played == 0 {
            if self.board.in_check() {
                return MATE as i32 + (ply as i32);
            } else {
                return STALEMATE as i32;
            }
        }

        let node_bound = if best_value as i32 >= beta {NodeBound::LowerBound}
            else if is_pv && !best_move.is_null() {NodeBound::Exact}
                else {NodeBound::UpperBound};

        tt_entry.place(zob, best_move, best_value as i16, pos_eval as i16, plys_to_zero as u8, node_bound);

        best_value
    }

    fn qsearch<N: PVNode>(&mut self, mut alpha: i32, beta: i32, max_depth: i32) -> i32 {
        unimplemented!()
    }

    fn main_thread(&self) -> bool {
        self.thread.id == 0
    }

    fn stop(&self) -> bool {
        self.thread.stop.load(Ordering::Relaxed)
    }

    pub fn print_startup(&self) {
        if self.use_stdout() {
            println!("info id {} start", self.thread.id);
        }
    }

    pub fn use_stdout(&self) -> bool {
        self.thread.use_stdout.load(Ordering::Relaxed)
    }

}

fn mvv_lva_sort(moves: &mut MoveList, board: &Board) {
    moves.sort_by_key(|a| {
        let piece = board.piece_at_sq((*a).get_src()).unwrap();

        if a.is_capture() {
            piece.value() - board.captured_piece(*a).unwrap().value()
        } else if a.is_castle() {
            1
        } else if piece == Piece::P {
            if a.is_double_push().0 {
                2
            } else {
                3
            }
        } else {
            4
        }
    })
}

fn correct_bound_eq(tt_value: i32, beta: i32, bound: NodeBound) -> bool {
    if tt_value as i32 >= beta {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}

fn correct_bound(tt_value: i32, val: i32, bound: NodeBound) -> bool {
    if tt_value as i32 > val {
        bound as u8 & NodeBound::LowerBound as u8 != 0
    } else {
        bound as u8 & NodeBound::UpperBound as u8 != 0
    }
}


fn futility_margin(depth: u16) -> i32 {
    depth as i32 * 150
}