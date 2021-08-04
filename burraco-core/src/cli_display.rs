use crate::actions::BurracoGame;
use crate::actions::DiscardAction;
use crate::actions::DrawAction;
use crate::actions::PlayAction;
use crate::actions::PlayAction::*;
use crate::model::BurracoState;
use crate::model::Card;
use crate::model::Cards;
use crate::model::Rank;
use crate::model::Rank::*;
use crate::model::Run;
use crate::model::RunType::*;
use crate::model::Suit;
use crate::model::Suit::*;

use std::fmt;

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            Clubs => "♣",
            Diamonds => "♦",
            Hearts => "♥",
            Spades => "♠",
            Jokers => "",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Two => write!(f, "2"),
            Numerical(num) => write!(f, "{}", num),
            Jack => write!(f, "J"),
            Queen => write!(f, "Q"),
            King => write!(f, "K"),
            Ace => write!(f, "A"),
            Joker => write!(f, "JK"),
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for card in self.iter() {
            write!(f, "{}, ", card)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for Run {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.run_type() {
            Sequence => write!(f, "Sequence: {} ({} p)", self.cards(), self.score()),
            Group => write!(f, "Group: {} ({} p)", self.cards(), self.score()),
        }
    }
}

impl fmt::Display for BurracoState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Burraco table:")?;
        for t in 0..self.num_teams {
            writeln!(f, "  Team {}", t)?;
            for p in 0..self.num_team_players {
                writeln!(
                    f,
                    "    Player {}-{}: {} cards",
                    t,
                    p,
                    self.teams[t].players[p].hand.len()
                )?;
            }
            writeln!(f, "    Runs played")?;
            for run in &self.teams[t].played_runs {
                writeln!(f, "    - {}", run)?;
            }
        }
        writeln!(f, "  Draw pile: {} cards", self.draw_pile.len())?;
        writeln!(f, "  Open pile: {}", self.open_pile)?;
        writeln!(f)?;
        writeln!(f, "  Pot 1: {} cards", self.pot1.len())?;
        writeln!(f, "  Pot 2: {} cards", self.pot2.len())?;
        writeln!(f, "Cards tot: {}", self.cards_total())?;
        let (team, player) = self.player_team_idxs[self.player_turn];
        writeln!(f, "Current round: {}", self.round)?;
        writeln!(f, "Current turn: team {} player {}", team, player)
    }
}

impl fmt::Display for BurracoGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.state())?;
        writeln!(f, "Scoreboard: {:?}", &self.scoreboard())?;
        writeln!(f, "Current phase: {:?}", &self.phase())?;
        writeln!(f, "Current hand: {}", self.current_player().hand)?;
        writeln!(f, "---")
    }
}

impl fmt::Display for DrawAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DrawAction::DrawOpen => write!(f, "Collect open pile"),
            DrawAction::DrawPile => write!(f, "Draw from hidden pile"),
        }
    }
}

impl fmt::Display for PlayAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartRun(run) => write!(f, "Start run - {}", run.cards()),
            AppendTop(run_idx, cards) => write!(f, "Append top, to {} - {}", run_idx, cards),
            AppendBottom(run_idx, cards) => write!(f, "Append bottom, to {} - {}", run_idx, cards),
            ReplaceWildcard(run_idx, wilcard_idx, card) => write!(
                f,
                "Replace wildcard, for {}: with {} - position {}",
                run_idx, card, wilcard_idx
            ),
            MoveCard(run_idx, from, to) => {
                write!(f, "Move card, with {} - from {} to {}", run_idx, from, to)
            }
            Noop => write!(f, "Play nothing"),
        }
    }
}

pub fn print_play_actions(actions: &[(PlayAction, i32)], runs: &[Run]) {
    println!("Available actions:");
    for (i, (action, _d_score)) in actions.iter().enumerate() {
        let mut extra = String::new();
        match &action {
            PlayAction::AppendTop(run_idx, _) => {
                extra.push_str(&format!(" - {}", runs[*run_idx].cards()))
            }
            PlayAction::AppendBottom(run_idx, _) => {
                extra.push_str(&format!(" - {}", runs[*run_idx].cards()))
            }
            PlayAction::MoveCard(run_idx, _from, _to) => {
                extra.push_str(&format!(" - {}", runs[*run_idx].cards()))
            }
            // TODO more?
            _ => {}
        };
        println!("  {}: {}{}", i, action, extra);
    }
}

impl fmt::Display for DiscardAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Discard {}", self.0)
    }
}
