use crate::model::Suit;
use crate::model::Suit::*;
use crate::model::Card;
use crate::model::Cards;
use crate::model::Run;
use crate::model::Run::*;
use crate::model::BurracoState;
use crate::model::Rank;
use crate::model::Rank::*;
use crate::actions::BurracoGame;

use std::fmt;

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            Clubs => "♣",
            Diamonds => "♦", 
            Hearts => "♥",
            Spades => "♠",
            Jokers => ""
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Two   => write!(f, "2"),
            Numerical(num) => write!(f, "{}", num),
            Jack  => write!(f, "J"),
            Queen => write!(f, "Q"),
            King  => write!(f, "K"),
            Ace   => write!(f, "A"),
            Joker => write!(f, "JK")
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
        match self {
            SequenceRun(cards) => write!(f, "Group: {}", cards),
            GroupRun(cards) => write!(f, "Sequence: {}", cards),
        }
    }
}

impl fmt::Display for BurracoState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Burraco table: \n")?;
        for t in 0..self.num_teams {
            write!(f, "  Team {}\n", t)?;
            for p in 0..self.num_team_players {
                write!(f, "    Player {}-{}: {} cards \n", t, p, self.teams[t].players[p].hand.len())?;
            }
            write!(f, "    Runs played\n")?;
            for run in &self.teams[t].played_runs {
                write!(f, "    - {}\n", run)?;
            }
        }
        write!(f, "  Draw pile: {} cards \n", self.draw_pile.len() )?;
        write!(f, "  Open pile: {} \n", self.open_pile )?;
        write!(f, "\n")?;
        write!(f, "  Pot 1: {} cards \n", self.pot1.len() )?;
        write!(f, "  Pot 2: {} cards \n", self.pot2.len() )?;
        let (team, player) = self.player_turn;
        write!(f, "Current turn: team {} player {} \n", team, player)
    }
}

impl fmt::Display for BurracoGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.state)?;
        write!(f, "Current phase: {:?} \n", &self.phase )?;
        write!(f, "Current hand: {} \n", self.state.current_player().hand )?;
        write!(f, "---")
    }
}