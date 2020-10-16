// search.rs contains the engine's search routine.

mod alpha_beta;
pub mod defs;
mod sorting;

use crate::{
    board::{defs::SQUARE_NAME, Board},
    defs::MAX_DEPTH,
    engine::defs::{ErrFatal, Information},
    movegen::MoveGenerator,
};
use crossbeam_channel::Sender;
use defs::{SearchControl, SearchInfo, SearchParams, SearchRefs, SearchTerminate, INF};
use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub struct Search {
    handle: Option<JoinHandle<()>>,
    control_tx: Option<Sender<SearchControl>>,
}

impl Search {
    pub fn new() -> Self {
        Self {
            handle: None,
            control_tx: None,
        }
    }

    pub fn init(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
        mg: Arc<MoveGenerator>,
    ) {
        // Set up a channel for incoming commands
        let (control_tx, control_rx) = crossbeam_channel::unbounded::<SearchControl>();

        // Create thread-local variables.
        let _t_report_tx = report_tx.clone();

        // Create the search thread.
        let h = thread::spawn(move || {
            // Pointer to Board and Move Generator for this thread.
            let arc_board = Arc::clone(&board);
            let arc_mg = Arc::clone(&mg);
            let mut search_info = SearchInfo::new();

            let mut quit = false;
            let mut halt = true;

            while !quit {
                let cmd = control_rx.recv().expect(ErrFatal::CHANNEL);

                match cmd {
                    SearchControl::Start => {
                        halt = false;
                    }
                    SearchControl::Stop => halt = true,
                    SearchControl::Quit => quit = true,
                    SearchControl::Nothing => (),
                }

                if !halt && !quit {
                    let mtx_board = arc_board.lock().expect(ErrFatal::LOCK);
                    let mut board = mtx_board.clone();
                    std::mem::drop(mtx_board);

                    search_info = SearchInfo::new();
                    search_info.termination = SearchTerminate::Nothing;

                    let mut search_params = SearchParams::new(7);
                    let mut search_refs = SearchRefs {
                        board: &mut board,
                        mg: &arc_mg,
                        search_params: &mut search_params,
                        search_info: &mut search_info,
                        control_rx: &control_rx,
                    };

                    Search::iterative_deepening(&mut search_refs);
                }

                match search_info.termination {
                    SearchTerminate::Stop => {
                        halt = true;
                    }
                    SearchTerminate::Quit => {
                        halt = true;
                        quit = true;
                    }
                    _ => (),
                }
            }
        });

        // Store the thread's handle and command sender.
        self.handle = Some(h);
        self.control_tx = Some(control_tx);
    }

    // This function is used to send commands into the search thread.
    pub fn send(&self, cmd: SearchControl) {
        if let Some(tx) = &self.control_tx {
            tx.send(cmd).expect(ErrFatal::CHANNEL);
        }
    }

    pub fn wait_for_shutdown(&mut self) {
        if let Some(h) = self.handle.take() {
            h.join().expect(ErrFatal::THREAD);
        }
    }
}

// Actual search routines.
impl Search {
    fn iterative_deepening(refs: &mut SearchRefs) {
        let mut depth = 1;
        let terminate = false;

        while depth <= refs.search_params.depth && depth < MAX_DEPTH && !terminate {
            let now = std::time::Instant::now();

            let eval = Search::alpha_beta(depth, -INF, INF, refs);

            let seconds = now.elapsed().as_millis() as f64 / 1000f64;
            let knodes = refs.search_info.nodes as f64 / 1000f64;
            let knps = (knodes / seconds).floor() as usize;

            println!(
                "depth: {}, best move: {}{}, eval: {}, nodes: {}, knps: {}",
                depth,
                SQUARE_NAME[refs.search_info.best_move.from()],
                SQUARE_NAME[refs.search_info.best_move.to()],
                eval,
                refs.search_info.nodes,
                knps
            );

            depth += 1;
        }
    }
}
