use super::{defs::ErrFatal, Engine};
use crate::{
    comm::{console::ConsoleReport, uci::UciReport, CommControl, CommReport, GeneralReport},
    evaluation::evaluate_position,
    search::defs::SearchControl,
};

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_reports(&mut self, comm_report: &CommReport) {
        // Split out the comm reports according to their source.
        match comm_report {
            CommReport::General(_) => self.comm_reports_general(comm_report),
            CommReport::Console(_) => self.comm_reports_console(comm_report),
            CommReport::Uci(_) => self.comm_reports_uci(comm_report),
        }
    }

    // Handles "General" Comm reports, that can be sent by any Comm module.
    fn comm_reports_general(&mut self, comm_report: &CommReport) {
        match comm_report {
            // Quit Comm, Search, and then the engine itself.
            CommReport::General(GeneralReport::Quit) => {
                self.comm.send(CommControl::Quit);
                self.search.send(SearchControl::Quit);
                self.quit = true;
            }

            // Print the Help screen for the Comm module.
            CommReport::General(GeneralReport::Help) => {
                self.comm.send(CommControl::PrintHelp);
                self.comm.send(CommControl::Update);
            }

            // Ignore if Nothing reported or report is Unknown.
            CommReport::General(GeneralReport::Nothing) => (),
            CommReport::General(GeneralReport::Unknown) => (),
            _ => (),
        }
    }

    // Handles "Console" Comm reports coming from the Console module.
    fn comm_reports_console(&mut self, comm_report: &CommReport) {
        match comm_report {
            // Execute the received move.
            CommReport::Console(ConsoleReport::Move(m)) => {
                self.execute_move(m.clone());
                self.comm.send(CommControl::Update);
            }

            // Send evaluation result upon request.
            CommReport::Console(ConsoleReport::Evaluate) => {
                let eval = evaluate_position(&self.board.lock().expect(ErrFatal::LOCK));
                self.comm.send(CommControl::PrintEvaluation(eval));
                self.comm.send(CommControl::Update);
            }

            CommReport::Console(ConsoleReport::Takeback) => {
                self.takeback_move();
                self.comm.send(CommControl::Update);
            }

            // Start or stop the search.
            CommReport::Console(ConsoleReport::Search) => self.search.send(SearchControl::Start),
            CommReport::Console(ConsoleReport::Cancel) => self.search.send(SearchControl::Stop),
            _ => (),
        }
    }

    // Handles "Uci" Comm reports sent by the UCI-module.
    fn comm_reports_uci(&mut self, comm_report: &CommReport) {
        match comm_report {
            // Received command: 'uci'
            CommReport::Uci(UciReport::Uci) => self.comm.send(CommControl::Identify),

            // Execute the received move.
            CommReport::Uci(UciReport::Move(m)) => {
                self.execute_move(m.clone());
                self.comm.send(CommControl::Update);
            }

            // Send evaluation result upon request.
            CommReport::Uci(UciReport::Evaluate) => {
                let eval = evaluate_position(&self.board.lock().expect(ErrFatal::LOCK));
                self.comm.send(CommControl::PrintEvaluation(eval));
                self.comm.send(CommControl::Update);
            }

            CommReport::Uci(UciReport::Takeback) => {
                self.takeback_move();
                self.comm.send(CommControl::Update);
            }

            // Start or stop the search.
            CommReport::Uci(UciReport::Search) => self.search.send(SearchControl::Start),
            CommReport::Uci(UciReport::Cancel) => self.search.send(SearchControl::Stop),
            _ => (),
        }
    }
}
