
use crate::model::BurracoState;
use crate::model::Card;
use crate::model::Cards;
use crate::model::Team;
use crate::model::Suit::*;
use crate::model::Rank::*;
use crate::model::Player;
use crate::model::Run;
use PlayAction::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Draw, 
    Play, 
    Discard,
    Finished(usize) // winning team
}

pub struct BurracoGame {
    state: BurracoState,
    phase: GamePhase
}

impl BurracoGame {
    pub fn from(state: BurracoState) -> BurracoGame {
        BurracoGame {
            state,
            phase: GamePhase::Draw
        }
    }

    pub fn current_player(&self) -> &Player {
        let (team, player) = self.state.player_turn;
        &self.state.teams[team].players[player]
    }

    pub fn current_team(&self) -> &Team {
        let (team, _) = self.state.player_turn;
        &self.state.teams[team]
    }

    pub fn state(&self) -> &BurracoState {
        &self.state
    }

    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    pub fn draw(&mut self, action: DrawAction) -> Result<(),String> {
        dbg!(&action);

        let (team, player) = self.state.player_turn;
        
        if self.phase != GamePhase::Draw {
            return Err(format!("Draw action invalid when phase is: {:?}", self.phase));
        }

        match action {
            DrawAction::DrawOpen => {
                let open_pile = &mut self.state.open_pile;
                self.state.teams[team].players[player].hand
                    .append(open_pile);

            },
            DrawAction::DrawPile => {
                let draw_pile = &mut self.state.draw_pile;
                self.state.teams[team].players[player].hand
                    .append(&mut draw_pile.drain_back(1));
            }
        }

        self.state.teams[team].players[player].hand.sort();
        if self.state.draw_pile.is_empty() {
            self.phase = GamePhase::Finished(999); // TODO calculate winner
            Ok( () )   
        } else {
            self.phase = GamePhase::Play;
            Ok( () )
        }
    }
    
    pub fn play(&mut self, action: PlayAction) -> Result<(),String> {
        use std::collections::HashSet;
        use std::iter::FromIterator;

        dbg!(&action);
        if self.phase != GamePhase::Play {
            return Err(format!("Play action invalid when phase is: {:?}", self.phase));
        }

        let (team, player) = self.state.player_turn;

        match action {
            PlayAction::Noop => {},
            PlayAction::StartRun(run) => {
                let uniq: HashSet<Card> = HashSet::from_iter(run.cards().iter().cloned());
                let all_contained_in_hand = uniq.iter().all(|c1| {
                    let run_count = run.cards().iter().filter(|c2| c1 == *c2).count();
                    let hand_count = self.current_player().hand.iter().filter(|c2| c1 == *c2).count();
                    hand_count >= run_count
                });
                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }
                
                
                let hand = &mut self.state.teams[team].players[player].hand;
                for card in run.cards().iter() {
                    let index = hand.iter().position(|c| c == card).expect("We have checked that run counts exceed hand counts");
                    hand.remove(index);
                }
                self.state.teams[team].played_runs.push(run);
            }
            _ => return Err(format!("Not implemented for {:?}", &action))
        }
        if self.current_player().hand.is_empty() {
            self.phase = GamePhase::Finished(999); // TODO: calculate winner 
        } else {
            self.phase = GamePhase::Discard;
        }
        
        Ok( () )
    }
    
    pub fn discard(&mut self, action: DiscardAction) -> Result<(),String>  {
        dbg!(&action);
        if self.phase != GamePhase::Discard {
            return Err(format!("Discard action invalid when phase is: {:?}", self.phase));
        }
        let (team, player) = self.state.player_turn;
        if let Some(index) = self.current_player().hand.iter().position(|c| *c == action.0) {
            self.state.teams[team].players[player].hand.remove(index);
            self.state.open_pile.push(action.0);
        } else {
            return Err(format!("Cannot discard card not in hand: {}", action.0));
        }

        if self.current_player().hand.is_empty() {
            self.phase = GamePhase::Finished(999); // TODO: calculate winner 
        } else {
            self.phase = GamePhase::Draw;

            let (mut team, mut player) = self.state.player_turn;
            let next_team = (team + 1) %  self.state.num_teams;
            if next_team == 0 {
                self.state.round += 1;
            }
            if next_team == 0 && (team + 1) * (player + 1) == self.state.num_teams * self.state.num_team_players {
                player = player + 1;
            }
            team = next_team;
            self.state.player_turn = (team, player);
        }

        Ok( () )
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawAction {
    DrawOpen,
    DrawPile
}

#[derive(Debug, Clone)]
pub enum PlayAction {
    StartRun(Run),
    AppendTop(usize, Cards), // run_idx
    AppendBottom(usize, Cards), // run_idx
    ReplaceWildcard(usize, usize), // run_idx
    MoveWildcard(usize, usize, usize), // run_idx, from, to
    Noop
}

impl PlayAction {
    pub fn enumerate(team_runs: &Vec<Run>, player_hand: &Cards) -> Vec<PlayAction> {
        let mut actions = vec![Noop];
        // start run actions
        // for now try all
        // TODO: write smarter that is not O(n^3) :)
        for i in 0..player_hand.len() {
            let card1 = player_hand[i];
            for j in 0..player_hand.len() {
                if i == j { continue };
                let card2 = player_hand[j];

                let card1_is_wildcard = card1.1 == Two || card1.1 == Joker ;
                let card2_is_wildcard = card1.1 == Two || card1.1 == Joker ;
                // break on different suite
                if !card1_is_wildcard && !card2_is_wildcard && card1.0 != card2.0 {
                    continue;
                }

                for k in 0..player_hand.len() {
                    if j == k || i == k { continue };
                    let card3 = player_hand[k];
                    let maybe_run = Run::build_sequence_run(
                        Cards(vec![card1, card2, card3]));
                    if let Ok(run) = maybe_run {
                        actions.push(PlayAction::StartRun(run));
                    }
                }
            }
        }
        dbg!(actions)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiscardAction(pub Card);