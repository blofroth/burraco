use crate::actions::PlayAction;
use crate::actions::DrawAction;
use crate::actions::DiscardAction;
use crate::model::BurracoState;
use crate::model::Cards;
use crate::cli_display::print_play_actions;

pub trait BurracoAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction;
    fn select_play_action(&mut self, actions: Vec<(PlayAction, i32)>, state: &BurracoState) -> PlayAction;
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

    fn select_play_action(&mut self, actions: Vec<(PlayAction, i32)>, _state: &BurracoState) -> PlayAction {
        actions.into_iter().last().unwrap().0
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        DiscardAction(hand[0])
    }

    fn display(&self) -> String {
        "Dumb agent".into()
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

    fn select_play_action(&mut self, actions: Vec<(PlayAction, i32)>, _state: &BurracoState) -> PlayAction {
        let max_action = actions.iter().max_by_key(|(_a, d_score)| d_score).expect("at least noop action");
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

use rand::Rng;
use rand::prelude::SliceRandom;
pub struct RandomAgent<R: Rng + ?Sized> {
    pub rng: R
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

    fn select_play_action(&mut self, actions: Vec<(PlayAction, i32)>, _state: &BurracoState) -> PlayAction {
        if actions.len() == 1 {
            actions[0].clone().0
        } else {
            (&actions[1..]).choose(&mut self.rng).expect("we know at least noop exists").clone().0
        }
        
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        DiscardAction(*hand.choose(&mut self.rng).expect("game would have ended if empty hand"))
    }

    fn display(&self) -> String {
        "Random agent".into()
    }
}

pub struct ManualCliAgent {}
use std::io;
use std::io::Write;

impl ManualCliAgent {
    fn display_concise_state(state: &BurracoState) {
        let (team, player) = state.player_team_idxs[state.player_turn];
        
        
        for (_, other_team) in state.teams.iter().enumerate().filter(|(i, _)| *i != team ) {
            println!("Other team:");
            print!(" h: ");
            for other_player in &other_team.players {
                print!("[{}] ", other_player.hand.len());
            }
            println!("");
            for run in &other_team.played_runs {
                println!(" r: {}", run);
            }
        }

        for (_, curr_team) in state.teams.iter().enumerate().filter(|(i, _)| *i == team ) {
            println!("Current team:");
            print!(" h: ");
            for team_player in &curr_team.players {
                print!("[{}] ", team_player.hand.len());
            }
            println!("");
            for run in &curr_team.played_runs {
                println!(" r: {}", run);
            }
        }

        println!("Piles: {} [{}] (pots: [{}] [{}])", state.open_pile, state.draw_pile.len(), state.pot1.len(), state.pot1.len());
        let hand = &state.teams[team].players[player].hand;
        println!("Hand: {}", hand);
    }
}

impl BurracoAgent for ManualCliAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction {
        println!("[{}]", self.display());
        ManualCliAgent::display_concise_state(state);
        println!("Select a draw action:");
        println!(" 0 - Draw from open pile");
        println!(" 1 - Draw from hidden pile");
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
                    continue;
                }
            };
            return action;
        }
    }

    fn select_play_action(&mut self, actions: Vec<(PlayAction, i32)>, state: &BurracoState) -> PlayAction {
        println!("[{}]", self.display());
        ManualCliAgent::display_concise_state(state);
        println!("Select a play action:");
        print_play_actions(&actions);
        println!("then press ENTER");

        let mut choice = String::new();

        loop {
            print!("> ");
            io::stdout().flush().expect("to flush ok");

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");
            println!();

            let choice_idx: usize = if let Ok(idx) = choice.trim().parse() { idx } else {
                println!("Please enter a valid action index (got: '{}')", choice);
                continue;
            };
            if choice_idx >= actions.len() {
                println!("Action index out of valid range (got: '{}')", choice_idx);
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

            let choice_idx: usize = if let Ok(idx) = choice.trim().parse() { idx } else {
                println!("Please enter a valid discard card index (got: '{}')", choice);
                continue;
            };
            if choice_idx >= hand.len() {
                println!("Discard card index  out of valid range (got: '{}')", choice_idx);
                continue;
            }
            
            return DiscardAction(hand[choice_idx].clone());
        }
    }

    fn display(&self) -> String {
        "Manual commandline agent".into()
    }
    
}

