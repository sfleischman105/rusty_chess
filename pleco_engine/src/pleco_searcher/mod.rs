pub mod misc;
pub mod options;
pub mod threads;
pub mod thread_search;
pub mod root_moves;

use pleco::tools::UCILimit;
use pleco::tools::tt::TranspositionTable;
use pleco::Board;
use pleco::BitMove;
use pleco::tools::timer::Timer;
use pleco::board::eval::*;

use std::thread;
use std::io;

use self::options::{UciOption,AllOptions,UciOptionMut};
use self::threads::ThreadPool;


const MAX_PLY: u16 = 126;
const THREAD_STACK_SIZE: usize = MAX_PLY as usize + 7;
pub const MAX_THREADS: usize = 256;
pub const DEFAULT_TT_SIZE: usize = 256;

lazy_static! {
    pub static ref TT_TABLE: TranspositionTable = TranspositionTable::new(DEFAULT_TT_SIZE);
}

pub static ID_NAME: &str = "Pleco";
pub static ID_AUTHORS: &str = "Stephen Fleischman";
pub static VERSION: &str = "0.0.3";

#[derive(PartialEq)]
enum SearchType {
    None,
    Search,
    Ponder,
}

pub struct PlecoSearcher {
    options: AllOptions,
    thread_pool: ThreadPool,
    search_mode: SearchType,
    board: Option<Board>,
    limit: Option<UCILimit>
}


impl PlecoSearcher {

    pub fn init(use_stdout: bool) -> Self {
        unsafe {
            TT_TABLE.clear();
        }
        let mut pool = ThreadPool::new();
        pool.stdout(use_stdout);
        pool.set_thread_count(8);
        PlecoSearcher {
            options: AllOptions::default(),
            thread_pool: pool,
            search_mode: SearchType::None,
            board: None,
            limit: None,
        }
    }

    pub fn uci(&mut self) {
        let mut full_command = String::new();
        'main: loop {
            full_command.clear();
            io::stdin().read_line(&mut full_command).ok().unwrap();
            let args: Vec<&str> = full_command.split_whitespace().collect();
            let command: &str = args.first().unwrap_or(&"");
            match command {
                "" => continue,
                "uci" => self.uci_startup(),
                "setoption" => self.apply_option(&full_command),
                "options" | "alloptions" => self.options.print_curr(),
                "ucinewgame" => self.clear_search(),
                "isready" => println!("readyok"),
                "position" => self.parse_position(&args[1..]),
                "go" => self.uci_go(&args[1..]),
                "quit" | "stop" => {
                    self.halt();
                    break;
                },
                _ => println!("Unknown Command: {}",full_command)
            }

        }
    }

    pub fn clear_search(&mut self) {

    }

    fn parse_position(&mut self, args: &[&str]) {
        let start: &str = args[0];
        self.board = if start == "startpos" {
            Some(Board::default())
        } else if start == "fen" {
            let fen_string: String = args[1..].iter()
                .take_while(|p: &&&str| **p != "moves")
                .map(|p| (*p).to_string())
                .collect::<Vec<String>>()
                .connect(" ");
            Board::new_from_fen(&fen_string).ok()
        } else {
            None
        };

        let mut moves_start: Option<usize> =  None;
        for (i, mov) in args.iter().enumerate() {
            if *mov == "moves" {
                moves_start = Some(i);
            }
        };

        if let Some(start) = moves_start {
            if let Some(ref mut board) = self.board {
                args[start..]
                    .iter()
                    .for_each(|p| {
                        board.apply_uci_move(*p);
                    })
            }
        }
    }

    fn uci_go(&mut self, args: &[&str]) {
        let mut token_idx: usize = 1;
//        while let Some(token) = *args[token_idx] {
//
//        }
    }

    fn apply_option(&mut self, option: &str) {
        let c = self.options.apply_option(option);
        match c {
            UciOptionMut::Button(c)   => {(c)(self);},
            UciOptionMut::Check(c, v) => {(c)(self, v);},
            UciOptionMut::Spin(c, v)  => {(c)(self, v);},
            UciOptionMut::Combo(c, v) => {(c)(self, v);},
            UciOptionMut::Text(c, v)  => {(c)(self, v);},
            UciOptionMut::None => {},
        }

    }

    fn uci_startup(&self) {
        println!("id name {}",ID_NAME);
        println!("id authors {}", ID_AUTHORS);
        self.options.print_all();
        println!("uciok");
    }

    pub fn search(&mut self, board: &Board, limit: &UCILimit) {
        TT_TABLE.new_search();
        self.search_mode = SearchType::Search;
        self.thread_pool.uci_search(&board, &limit);
    }

    pub fn halt(&mut self) {
        self.thread_pool.stop_searching();
        self.search_mode = SearchType::None;
    }

    pub fn stop_search_get_move(&mut self) -> BitMove {
        if self.is_searching() {
            self.halt();
            return self.thread_pool.get_move();
        } else {
            return BitMove::null();
        }
    }

    pub fn is_searching(&self) -> bool {
        if self.search_mode == SearchType::None {
            return false;
        }
        true
    }

    pub fn hash_percent(&self) -> f64 {
        TT_TABLE.hash_percent()
    }

    pub fn clear_tt(&mut self) {
        unsafe {TT_TABLE.clear() };
    }

    pub fn resize_tt(&mut self, mb: usize) {
        unsafe {TT_TABLE.resize_to_megabytes(mb)};
    }

    pub fn use_stdout(&mut self, stdout: bool) {
        self.thread_pool.stdout(stdout);
    }


}

//fn parse_board_position(tokens: Vec<String>) -> Board {
//    let mut token_stack = tokens.clone();
//    token_stack.reverse();
//    token_stack.pop();
//
//    let start_str = token_stack.pop().unwrap();
//    let start = &start_str;
//    let mut board = if start == "startpos" {
//        Some(Board::default())
//    } else if start == "fen" {
//        let fen_string: &str = &token_stack.pop().unwrap();
//        Board::new_from_fen(fen_string).ok()
//    } else {
//        panic!()
//    };
//
//    if !token_stack.is_empty() {
//        let next = &token_stack.pop().unwrap();
//        if next == "moves" {
//            while !token_stack.is_empty() {
//                let bit_move = &token_stack.pop().unwrap();
//                let mut all_moves: Vec<BitMove> = board.generate_moves();
//                'check_legality: loop {
//                    if all_moves.is_empty() {
//                        panic!();
//                    }
//                    let curr_move: BitMove = all_moves.pop().unwrap();
//                    if &curr_move.stringify() == bit_move {
//                        board.apply_move(curr_move);
//                        break 'check_legality
//                    }
//                }
//            }
//        }
//    }
//    board
//}
//
//fn parse_limit(tokens: Vec<String>) -> UCILimit {
//    let mut token_stack = tokens.clone();
//    token_stack.reverse();
//
//    let mut white_time: i64 = i64::max_value();
//    let mut black_time: i64 = i64::max_value();
//    let mut white_inc: i64 = i64::max_value();
//    let mut black_inc: i64 = i64::max_value();
//
//    while !token_stack.is_empty() {
//        let token = token_stack.pop().unwrap();
//        if token == "inf" {
//            return UCILimit::Infinite;
//        } else if token == "wtime" {
//            white_time = unwrap_val_or(&mut token_stack, i64::max_value());
//        } else if token == "btime" {
//            black_time = unwrap_val_or(&mut token_stack, i64::max_value());
//        } else if token == "winc" {
//            white_inc = unwrap_val_or(&mut token_stack, 0);
//        } else if token == "binc" {
//            black_inc = unwrap_val_or(&mut token_stack, 0);
//        } else if token == "depth" {
//            return UCILimit::Depth(token_stack.pop().unwrap().parse::<u16>().unwrap());
//        } else if token == "mate" {
//            unimplemented!()
//        } else if token == "nodes" {
//            unimplemented!()
//        } else if token == "movestogo" {
//            unimplemented!()
//        } else if token == "movetime" {
//            unimplemented!()
//        }
//    }
//    UCILimit::Time(
//        Timer::new(white_time, black_time, white_inc, black_inc)
//    )
//}


#[cfg(test)]
mod tests {

    use super::*;

//    #[test]
    pub fn testme() {
        {
            let mut s = PlecoSearcher::init(false);
            let limit = UCILimit::Infinite;
            let board = Board::default();
            s.search(&board, &limit);
            thread::sleep_ms(20000);
            s.stop_search_get_move();
            println!("TT Hash {}", 100.0 * TT_TABLE.hash_percent());
        }
    }

}
