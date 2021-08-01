
use crate::model::BurracoState;
use crate::model::Card;
use crate::model::Cards;
use crate::model::Team;
use crate::model::Rank::*;
use crate::model::Player;
use crate::model::Run;
use crate::model::RunType;
use crate::model::Append;
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
    
    fn cards_in_hand(cards: &Cards, player: &Player) -> bool {
        use std::collections::HashSet;
        use std::iter::FromIterator;
        let uniq: HashSet<Card> = HashSet::from_iter(cards.iter().cloned());

        uniq.iter().all(|c1| {
            let run_count = cards.iter().filter(|c2| c1 == *c2).count();
            let hand_count = player.hand.iter().filter(|c2| c1 == *c2).count();
            hand_count >= run_count
        })
    }

    fn remove_from_hand(player: &mut Player, cards: &Cards) -> Result<(), String> {
        let hand = &mut player.hand;
        for card in cards.iter() {
            let index = hand.iter().position(|c| c == card).ok_or("Cannot remove card that does not exist".to_string())?;
            hand.remove(index);
        }
        Ok( () )
    }

    pub fn play(&mut self, action: PlayAction) -> Result<(),String> {
        if self.phase != GamePhase::Play {
            return Err(format!("Play action invalid when phase is: {:?}", self.phase));
        }

        let (team, player) = self.state.player_turn;

        match action {
            Noop => { self.phase = GamePhase::Discard },
            StartRun(run) => {
                let all_contained_in_hand = BurracoGame::cards_in_hand(run.cards(), self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }
                
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], run.cards())?;
                self.state.teams[team].played_runs.push(run);
            }
            AppendTop(run_idx, cards) => {
                let all_contained_in_hand = BurracoGame::cards_in_hand(&cards, self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                let run = &self.state.teams[team].played_runs.get(run_idx).ok_or("Non-existing run index".to_string())?;
                let new_run= run.append(&cards, Append::Top)?;
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], &cards)?;
                self.state.teams[team].played_runs[run_idx] = new_run;

            },
            AppendBottom(run_idx, cards) => {
                let all_contained_in_hand = BurracoGame::cards_in_hand(&cards, self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                let run = &self.state.teams[team].played_runs.get(run_idx).ok_or("Non-existing run index".to_string())?;
                let new_run= run.append(&cards, Append::Bottom)?;
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], &cards)?;
                self.state.teams[team].played_runs[run_idx] = new_run;

            },
            ReplaceWildcard(run_idx, at, card) => {
                let cards = Cards(vec![card]);
                let all_contained_in_hand = BurracoGame::cards_in_hand(&cards, self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                let run = &self.state.teams[team].played_runs.get(run_idx).ok_or("Non-existing run index".to_string())?;
                let new_run = run.replace_wildcard(at, &card)?;
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], &cards)?;
                self.state.teams[team].played_runs[run_idx] = new_run;

            },
            MoveCard(run_idx, from, to) => {
                let run = &self.state.teams[team].played_runs.get(run_idx).ok_or("Non-existing run index".to_string())?;
                let new_run = run.move_card(from, to)?;
                self.state.teams[team].played_runs[run_idx] = new_run;
            },
            _ => todo!()
        }
        if self.current_player().hand.is_empty() {
            self.phase = GamePhase::Finished(999); // TODO: calculate winner 
        } 
        // else continue in draw, if not set to Discard by noop action
        
        // TODO update score?
        Ok( () )
    }
    
    pub fn discard(&mut self, action: DiscardAction) -> Result<(),String>  {
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
    /// run_idx, cards to append
    AppendTop(usize, Cards),
    /// run_idx, cards to append
    AppendBottom(usize, Cards), 
    /// run_idx, at, with card
    ReplaceWildcard(usize, usize, Card), 
    /// run_idx, from, to
    MoveCard(usize, usize, usize),
    Noop
}

impl PlayAction {
    pub fn enumerate(team_runs: &Vec<Run>, player_hand: &Cards, moves_allowed: usize) -> Vec<PlayAction> {
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
                
                if !card1_is_wildcard && !card2_is_wildcard {
                    if card1.0 != card2.0 {
                        // break on different suite
                        continue;
                    }
                    if card1.1 != Ace && card1.1.index() != card2.1.index() - 1 {
                        // break on bad seq
                        continue;
                    }
                }

                for k in 0..player_hand.len() {
                    if j == k || i == k { continue };
                    #[cfg(debug_assertions)]
                    eprintln!("p-start");

                    let card3 = player_hand[k];
                    let maybe_run = Run::build_sequence_run(
                        Cards(vec![card1, card2, card3]));
                    if let Ok(run) = maybe_run {
                        #[cfg(debug_assertions)]
                        eprintln!("p-start-ok");
                        actions.push(PlayAction::StartRun(run));
                    }
                }
            }
        }

        // append actions 
        for i in 0..player_hand.len() {
            let card = Cards(vec![player_hand[i]]);
            for (j, run) in team_runs.iter().enumerate() {
                #[cfg(debug_assertions)]
                eprintln!("p-append");
                if let Ok(_) = run.append(&card, Append::Top) {
                    #[cfg(debug_assertions)]
                    eprintln!("p-append-ok");
                    actions.push(PlayAction::AppendTop(j, card.clone()));
                }
                if let Ok(_) = run.append(&card, Append::Bottom) {
                    #[cfg(debug_assertions)]
                    eprintln!("p-append-ok");
                    actions.push(PlayAction::AppendBottom(j, card.clone()));
                }
            }
        }

        // replace wilcard actions
        for (i, run) in team_runs.iter().enumerate() {
            for j in 0..player_hand.len() {
                let card = player_hand[j]; 
                if card.1 == Joker {
                    continue;
                }
                for k in 0..run.cards().len() {
                    let rank_replace = run.cards()[k].1;
                    if rank_replace != Joker && rank_replace != Two {
                        continue;
                    }
                    #[cfg(debug_assertions)]
                    eprintln!("p-replace");
                    if let Ok(_) = run.replace_wildcard(k, &card) {
                        #[cfg(debug_assertions)]
                        eprintln!("p-replace-ok");
                        actions.push(PlayAction::ReplaceWildcard(i, k, card));
                    }
                }
            }
        }

        // move card actions
        if moves_allowed > 0 {
            for (i, run) in team_runs.iter().enumerate() {
                for from in 0..run.cards().len() {
                    let rank_move = run.cards()[from].1;
                    if rank_move != Joker && rank_move != Two && rank_move != Ace {
                        continue;
                    }
    
                    for to in 0..run.cards().len() {
                        #[cfg(debug_assertions)]
                        eprintln!("p-move");
                        if let Ok(_) = run.move_card(from, to) {
                            #[cfg(debug_assertions)]
                            eprintln!("p-move-ok");
                            actions.push(PlayAction::MoveCard(i, from, to));
                        }
                    }
                }
            }
        }

        actions
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DiscardAction(pub Card);