# Pleco

Pleco is a chess Engine & Library inspired by Stockfish, written entirely in Rust.

[![Pleco crate](https://img.shields.io/crates/v/pleco.svg)](https://crates.io/crates/pleco)
[![Pleco crate](https://img.shields.io/crates/v/pleco_engine.svg)](https://crates.io/crates/pleco_engine)
[![Build Status](https://api.travis-ci.org/sfleischman105/Pleco.svg?branch=master)](https://travis-ci.org/sfleischman105/Pleco)
[![Coverage Status](https://coveralls.io/repos/github/sfleischman105/Pleco/badge.svg?branch=master)](https://coveralls.io/github/sfleischman105/Pleco?branch=master)


This project is split into two crates, `pleco`, which contains the library functionality, and `pleco_engine`, which contains the
UCI (Universal Chess Interface) compatible Engine & AI. 

The overall goal for this project is to utilize the efficiency of Rust to create a Chess AI matching the speed of modern chess engines.

- [Documentation](https://docs.rs/pleco), [crates.io](https://crates.io/crates/pleco) for library functionality
- [Documentation](https://docs.rs/pleco_engine), [crates.io](https://crates.io/crates/pleco_engine) for UCI Engine and Advanced Searching functionality.

Planned & Implemented features
-------


The Library aims to have the following features upon completion
- [x] Bitboard Representation of Piece Locations:
- [x] Ability for concurrent Board State access, for use by parallel searchers
- [x] Full Move-generation Capabilities, including generation of pseudo-legal moves
- [x] Statically computed lookup-tables (including Magic Bitboards)
- [x] Zobrist Hashing
- [ ] PGN Parsing

The AI Bot aims to have the following features:
- [x] Alpha-Beta pruning
- [x] Multi-threaded search with rayon.rs
- [x] Queiscience-search
- [x] MVV-LVA sorting
- [x] Iterative Deepening
- [x] Aspiration Windows
- [x] Futility Pruning
- [x] Transposition Tables
- [ ] Null Move Heuristic
- [ ] Killer Moves

Standalone Installation and Use
-------

To use pleco as an executable, please [navigate to here](https://github.com/sfleischman105/Pleco/tree/master/pleco_engine) and read the `README.md`. 


Using Pleco as a Library
-------

To use Pleco inside your own Rust projects, [Pleco.rs is available as a library on crates.io.](https://crates.io/crates/pleco) Simply include the following in your Cargo.toml:

```
[dependencies]
pleco = "0.3.6"
```

And add the following to a `main.rs` or `lib.rs`:
```rust
extern crate pleco;
```

### Basic Usage
Setting up a board position is extremely simple.
```rust
use pleco::{Board,Player,Piece};

let board = Board::default();
assert_eq!(board.count_piece(Player::White,Piece::P), 8);
assert_eq!(&board.get_fen(),"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
```

#### Creating a board from a Position
A `Board` can be created with any valid chess position using a valid FEN (Forsyth-Edwards Notation) String. 
Check out the [Wikipedia article](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation) for more information on FEN Strings
and their format.

```rust
let board = Board::new_from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2");
```

#### Applying and Generating Moves
Moves are represented with a `BitMove` structure. They must be generated by a `Board` object directly, to be 
considered a valid move. Using `Board::generate_moves()` will generate all legal `BitMove`s of the current 
position for the current player.
```rust
use pleco::{Board,BitMove};

let mut board = Board::default(); // create a board of the starting position
let moves = board.generate_moves(); // generate all possible legal moves
board.apply_move(moves[0]);
assert_eq!(board.moves_played(), 1);
```


We can ask the Board to apply a move to itself from a string. This string must follow the format of a standard
UCI Move, in the format [src_sq][dst_sq][promo]. E.g., moving a piece from A1 to B3 would have a uci string of "a1b3",
while promoting a pawn would look something like "e7e81". If the board is supplied a UCI move that is either 
incorrectly formatted or illegal, false shall be returned.
```rust
let mut board = Board::default(); // create a board of the starting position
let success = board.apply_uci_move("e7e8q"); // apply a move where piece on e7 -> eq, promotes to queen
assert!(!success); // Wrong, not a valid move for the starting position
```

#### Undoing Moves
We can revert to the previous chessboard state with a simple Board::undo_move()
```rust
let mut board = Board::default();
board.apply_uci_move("e2e4"); // A very good starting move, might I say
assert_eq!(board.moves_played(),1);
board.undo_move();
assert_eq!(board.moves_played(),0);
```

For more informaton about `pleco` as a library, see the [pleco README.md](https://github.com/sfleischman105/Pleco/tree/master/pleco).

Contributing
-------

Any and all contributions are welcome! Open up a PR to contribute some improvements. Look at the Issues tab to see what needs some help. 


  
License
-------
Pleco is distributed under the terms of the MIT license. See LICENSE-MIT for details. Opening a pull requests is assumed to signal agreement with these licensing terms.