use crate::actions::DiscardAction;
use crate::actions::DrawAction;
use crate::actions::PlayAction;
use crate::cli_display::print_play_actions;
use crate::model::BurracoState;
use crate::model::Cards;
use crate::model::Rank;
use crate::model::RunType;

pub trait BurracoAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction;
    fn select_play_action(
        &mut self,
        actions: Vec<(PlayAction, i32)>,
        state: &BurracoState,
    ) -> PlayAction;
    fn select_discard_action(&mut self, hand: &Cards, state: &BurracoState) -> DiscardAction;
    fn display(&self) -> String;
}

pub struct DumbAgent {}

impl BurracoAgent for DumbAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction {
        let draw_action = if state.round % 2 == 0 {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        draw_action
    }

    fn select_play_action(
        &mut self,
        actions: Vec<(PlayAction, i32)>,
        _state: &BurracoState,
    ) -> PlayAction {
        actions.into_iter().last().unwrap().0
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        DiscardAction(hand[0])
    }

    fn display(&self) -> String {
        "Dumb agent".into()
    }
}

pub struct SmartAgent {}

impl SmartAgent {
    fn play_action_preference(action: &PlayAction) -> usize {
        match action {
            PlayAction::ReplaceWildcard(_, _, _) => 0,
            PlayAction::StartRun(run)
                if run.run_type() == RunType::Sequence
                    && run
                        .cards()
                        .iter()
                        .all(|c| c.1 != Rank::Joker && c.1 != Rank::Two) =>
            {
                10
            }
            PlayAction::StartRun(run) if run.run_type() == RunType::Sequence => 10,
            PlayAction::StartRun(run) if run.run_type() == RunType::Group => 15,
            PlayAction::StartRun(_run) => 16, // not needed?
            PlayAction::AppendTop(_, _) => 20,
            PlayAction::AppendBottom(_, _) => 20,
            PlayAction::Noop => 30,
            PlayAction::MoveCard(_, _, _) => 999,
        }
    }
}

impl BurracoAgent for SmartAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction {
        // TODO smarter draw?
        let draw_action = if state.round % 2 == 0 {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        draw_action
    }

    fn select_play_action(
        &mut self,
        actions: Vec<(PlayAction, i32)>,
        _state: &BurracoState,
    ) -> PlayAction {
        let mut actions = actions;
        actions.sort_by_key(|(a, d_score)| (SmartAgent::play_action_preference(a), *d_score));
        actions.into_iter().next().unwrap().0
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        // TODO smarter discard?
        DiscardAction(hand[0])
    }

    fn display(&self) -> String {
        "Smart agent".into()
    }
}

pub struct MaxAgent {}

impl BurracoAgent for MaxAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction {
        // TODO: some max calculation here of gain?
        let draw_action = if state.round % 2 == 0 {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        draw_action
    }

    fn select_play_action(
        &mut self,
        actions: Vec<(PlayAction, i32)>,
        _state: &BurracoState,
    ) -> PlayAction {
        let max_action = actions
            .iter()
            .max_by_key(|(_a, d_score)| d_score)
            .expect("at least noop action");
        max_action.clone().0
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        // TODO: some max calculation here of gain?
        DiscardAction(hand[0])
    }

    fn display(&self) -> String {
        "Max action score agent".into()
    }
}

use rand::prelude::SliceRandom;
use rand::Rng;
use rand::prelude::ThreadRng;
use rand::thread_rng;


pub struct RandomAgent<R: Rng + ?Sized> {
    pub rng: R,
}

impl<R: Rng + ?Sized> RandomAgent<R> {
    pub fn new_thread_rng() -> RandomAgent<ThreadRng> {
        RandomAgent{ rng: thread_rng() }
    }
}

impl<R: Rng + ?Sized> BurracoAgent for RandomAgent<R> {
    fn select_draw_action(&mut self, _state: &BurracoState) -> DrawAction {
        let draw_action = if self.rng.gen::<bool>() {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        draw_action
    }

    fn select_play_action(
        &mut self,
        actions: Vec<(PlayAction, i32)>,
        _state: &BurracoState,
    ) -> PlayAction {
        if actions.len() == 1 {
            actions[0].clone().0
        } else {
            (&actions[1..])
                .choose(&mut self.rng)
                .expect("we know at least noop exists")
                .clone()
                .0
        }
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        DiscardAction(
            *hand
                .choose(&mut self.rng)
                .expect("game would have ended if empty hand"),
        )
    }

    fn display(&self) -> String {
        "Random agent".into()
    }
}

pub struct ManualCliAgent {}
use std::io;
use std::io::Write;

pub struct ConciseStateView {
    state: BurracoState,
}

use std::fmt;

impl fmt::Display for ConciseStateView {
    fn fmt(&self, w: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (team, player) = self.state.player_team_idxs[self.state.player_turn];

        for (_, other_team) in self
            .state
            .teams
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != team)
        {
            writeln!(w, "Other team:")?;
            write!(w, " h: ")?;
            for other_player in &other_team.players {
                write!(w, "[{}] ", other_player.hand.len())?;
            }
            writeln!(w, "")?;
            for run in &other_team.played_runs {
                writeln!(w, " r: {}", run)?;
            }
        }

        for (_, curr_team) in self
            .state
            .teams
            .iter()
            .enumerate()
            .filter(|(i, _)| *i == team)
        {
            writeln!(w, "Current team:")?;
            write!(w, " h: ")?;
            for team_player in &curr_team.players {
                print!("[{}] ", team_player.hand.len());
            }
            writeln!(w, "")?;
            for (i, run) in curr_team.played_runs.iter().enumerate() {
                writeln!(w, " r[{}]: {}", i, run)?;
            }
        }

        writeln!(
            w,
            "Piles: <0>{} <1>[{}] (pots: [{}] [{}])",
            self.state.open_pile,
            self.state.draw_pile.len(),
            self.state.pot1.len(),
            self.state.pot1.len()
        )?;
        let hand = &self.state.teams[team].players[player].hand;
        writeln!(w, "Hand: {}", hand)?;
        writeln!(
            w,
            "       {}",
            hand.iter()
                .enumerate()
                .map(|(i, _c)| format!("{{{:width$}}}", i, width = 2))
                .collect::<String>()
        )
    }
}

impl ManualCliAgent {
    fn display_concise_state(state: &BurracoState) {
        println!("{}", ManualCliAgent::concise_state(state));
    }

    pub fn concise_state(state: &BurracoState) -> ConciseStateView {
        ConciseStateView {
            state: state.clone(),
        }
    }
}

impl BurracoAgent for ManualCliAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction {
        println!("[{}]", self.display());
        ManualCliAgent::display_concise_state(state);
        println!("Select a draw action:");
        println!(" <0> - Draw from open pile");
        println!(" <1> - Draw from hidden pile");
        println!("then press ENTER");

        let mut choice = String::new();

        loop {
            print!("> ");
            io::stdout().flush().expect("to flush ok");
            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");
            println!();

            let action = match &choice.trim()[..] {
                "0" => DrawAction::DrawOpen,
                "1" => DrawAction::DrawPile,
                _ => {
                    println!("Invalid draw action choice: {}", choice);
                    choice.clear();
                    continue;
                }
            };
            return action;
        }
    }

    fn select_play_action(
        &mut self,
        actions: Vec<(PlayAction, i32)>,
        state: &BurracoState,
    ) -> PlayAction {
        println!("[{}]", self.display());
        ManualCliAgent::display_concise_state(state);
        println!("Select a play action:");
        print_play_actions(&actions, &state.teams[state.curr_team()].played_runs);
        println!("then press ENTER");

        let mut choice = String::new();

        loop {
            print!("> ");
            io::stdout().flush().expect("to flush ok");

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");
            println!();

            let choice_idx: usize = if let Ok(idx) = choice.trim().parse() {
                idx
            } else {
                println!("Please enter a valid action index (got: '{}')", choice);
                choice.clear();
                continue;
            };
            if choice_idx >= actions.len() {
                println!("Action index out of valid range (got: '{}')", choice_idx);
                choice.clear();
                continue;
            }

            return actions[choice_idx].clone().0;
        }
    }

    fn select_discard_action(&mut self, hand: &Cards, state: &BurracoState) -> DiscardAction {
        println!("[{}]", self.display());
        ManualCliAgent::display_concise_state(state);
        println!("Select a discard action:");
        for (idx, card) in hand.iter().enumerate() {
            println!(" {} - {}", idx, card);
        }
        println!("then press ENTER");

        let mut choice = String::new();

        loop {
            print!("> ");
            io::stdout().flush().expect("to flush ok");
            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");
            println!();

            let choice_idx: usize = if let Ok(idx) = choice.trim().parse() {
                idx
            } else {
                println!(
                    "Please enter a valid discard card index (got: '{}')",
                    choice
                );
                choice.clear();
                continue;
            };
            if choice_idx >= hand.len() {
                println!(
                    "Discard card index  out of valid range (got: '{}')",
                    choice_idx
                );
                choice.clear();
                continue;
            }

            return DiscardAction(hand[choice_idx].clone());
        }
    }

    fn display(&self) -> String {
        "Manual commandline agent".into()
    }
}
