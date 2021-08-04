use std::usize;

use crate::model::Append;
use crate::model::BurracoState;
use crate::model::Card;
use crate::model::Cards;
use crate::model::Player;
use crate::model::Rank::*;
use crate::model::Run;
use crate::model::RunType;
use crate::model::Team;
use PlayAction::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Draw,
    Play,
    Discard,
    Finished(usize), // winning team
}

pub struct BurracoGame {
    state: BurracoState,
    phase: GamePhase,
}

impl BurracoGame {
    pub fn from(state: BurracoState) -> BurracoGame {
        BurracoGame {
            state,
            phase: GamePhase::Draw,
        }
    }

    pub fn current_player(&self) -> &Player {
        let (team, player) = self.state.player_team_idxs[self.state.player_turn];
        &self.state.teams[team].players[player]
    }

    pub fn current_team(&self) -> &Team {
        let (team, _player) = self.state.player_team_idxs[self.state.player_turn];
        &self.state.teams[team]
    }

    pub fn state(&self) -> &BurracoState {
        &self.state
    }

    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    pub fn draw(&mut self, action: DrawAction) -> Result<(), String> {
        let (team, player) = self.state.player_team_idxs[self.state.player_turn];

        if self.phase != GamePhase::Draw {
            return Err(format!(
                "Draw action invalid when phase is: {:?}",
                self.phase
            ));
        }

        match action {
            DrawAction::DrawOpen => {
                let open_pile = &mut self.state.open_pile;
                self.state.teams[team].players[player]
                    .hand
                    .append(open_pile);
            }
            DrawAction::DrawPile => {
                let draw_pile = &mut self.state.draw_pile;
                self.state.teams[team].players[player]
                    .hand
                    .append(&mut draw_pile.drain_back(1));
            }
        }

        self.state.teams[team].players[player].hand.sort();
        if self.state.draw_pile.is_empty() {
            self.phase = GamePhase::Finished(self.winning_team()); // TODO calculate winner
            Ok(())
        } else {
            self.phase = GamePhase::Play;
            Ok(())
        }
    }

    fn cards_in_hand(cards: &Cards, player: &Player) -> bool {
        use std::collections::HashSet;
        let uniq: HashSet<Card> = cards.iter().cloned().collect::<HashSet<_>>();

        uniq.iter().all(|c1| {
            let run_count = cards.iter().filter(|c2| c1 == *c2).count();
            let hand_count = player.hand.iter().filter(|c2| c1 == *c2).count();
            hand_count >= run_count
        })
    }

    fn remove_from_hand(player: &mut Player, cards: &Cards) -> Result<(), String> {
        let hand = &mut player.hand;
        for card in cards.iter() {
            let index = hand
                .iter()
                .position(|c| c == card)
                .ok_or("Cannot remove card that does not exist".to_string())?;
            hand.remove(index);
        }
        Ok(())
    }

    pub fn play(&mut self, action: PlayAction) -> Result<(), String> {
        if self.phase != GamePhase::Play {
            return Err(format!(
                "Play action invalid when phase is: {:?}",
                self.phase
            ));
        }

        let (team, player) = self.state.player_team_idxs[self.state.player_turn];

        match action {
            Noop => self.phase = GamePhase::Discard,
            StartRun(run) => {
                let all_contained_in_hand =
                    BurracoGame::cards_in_hand(run.cards(), self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                BurracoGame::remove_from_hand(
                    &mut self.state.teams[team].players[player],
                    run.cards(),
                )?;
                self.state.teams[team].played_runs.push(run);
            }
            AppendTop(run_idx, cards) => {
                let all_contained_in_hand =
                    BurracoGame::cards_in_hand(&cards, self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                let run = &self.state.teams[team]
                    .played_runs
                    .get(run_idx)
                    .ok_or("Non-existing run index".to_string())?;
                let new_run = run.append(&cards, Append::Top)?;
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], &cards)?;
                self.state.teams[team].played_runs[run_idx] = new_run;
            }
            AppendBottom(run_idx, cards) => {
                let all_contained_in_hand =
                    BurracoGame::cards_in_hand(&cards, self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                let run = &self.state.teams[team]
                    .played_runs
                    .get(run_idx)
                    .ok_or("Non-existing run index".to_string())?;
                let new_run = run.append(&cards, Append::Bottom)?;
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], &cards)?;
                self.state.teams[team].played_runs[run_idx] = new_run;
            }
            ReplaceWildcard(run_idx, at, card) => {
                let cards = Cards(vec![card]);
                let all_contained_in_hand =
                    BurracoGame::cards_in_hand(&cards, self.current_player());

                if !all_contained_in_hand {
                    return Err("Cannot place cards not in hand!".into());
                }

                let run = &self.state.teams[team]
                    .played_runs
                    .get(run_idx)
                    .ok_or("Non-existing run index".to_string())?;
                let new_run = run.replace_wildcard(at, &card)?;
                BurracoGame::remove_from_hand(&mut self.state.teams[team].players[player], &cards)?;
                self.state.teams[team].played_runs[run_idx] = new_run;
            }
            MoveCard(run_idx, from, to) => {
                if from == to {
                    return Err("No use moving card to same index".into());
                }
                let run = &self.state.teams[team]
                    .played_runs
                    .get(run_idx)
                    .ok_or("Non-existing run index".to_string())?;
                let new_run = run.move_card(from, to)?;
                self.state.teams[team].played_runs[run_idx] = new_run;
            }
        }

        self.state.teams[team]
            .played_runs
            .sort_by_key(|r| (r.run_type() != RunType::Sequence, r.cards().len()));

        if self.current_player().hand.is_empty() {
            // TODO prevent action where game ends without team burraco
            if !self.current_team().has_reached_pot {
                if !self.state.pot1.is_empty() {
                    // "discard and get pot", can only play next turn
                    self.state.teams[team].players[player]
                        .hand
                        .append(&mut self.state.pot1.drain_back(11));
                    self.state.teams[team].has_reached_pot = true;
                } else if !self.state.pot2.is_empty() {
                    // "discard and get pot", can only play next turn
                    self.state.teams[team].players[player]
                        .hand
                        .append(&mut self.state.pot1.drain_back(11));
                    self.state.teams[team].has_reached_pot = true;
                } else {
                    // both pots taken, game ends (should really happen in other top else clause?)
                    self.phase = GamePhase::Finished(self.winning_team());
                }
            } else {
                // team has already reached one pot
                // TODO: game should end, I think
                self.phase = GamePhase::Finished(self.winning_team());
            }
        }

        // else continue in draw for next player, if not set to Discard by noop action

        // TODO update score?
        Ok(())
    }

    pub fn discard(&mut self, action: DiscardAction) -> Result<(), String> {
        if self.phase != GamePhase::Discard {
            return Err(format!(
                "Discard action invalid when phase is: {:?}",
                self.phase
            ));
        }
        let (team, player) = self.state.player_team_idxs[self.state.player_turn];
        if let Some(index) = self
            .current_player()
            .hand
            .iter()
            .position(|c| *c == action.0)
        {
            self.state.teams[team].players[player].hand.remove(index);
            self.state.open_pile.push(action.0);
        } else {
            return Err(format!("Cannot discard card not in hand: {}", action.0));
        }

        if self.current_player().hand.is_empty() {
            // TODO code duplication with play check?
            // TODO more sophisticated "reached/used" pot check?
            // TODO prevent action where game ends without team burraco
            if !self.current_team().has_reached_pot {
                if !self.state.pot1.is_empty() {
                    // "flying pot", can continue playing
                    self.state.teams[team].players[player]
                        .hand
                        .append(&mut self.state.pot1.drain_back(11));
                    self.state.teams[team].players[player].hand.sort();
                    self.state.teams[team].has_reached_pot = true;
                    self.phase = GamePhase::Draw;
                } else if !self.state.pot2.is_empty() {
                    // "flying pot", can continue playing
                    self.state.teams[team].players[player]
                        .hand
                        .append(&mut self.state.pot2.drain_back(11));
                    self.state.teams[team].players[player].hand.sort();
                    self.state.teams[team].has_reached_pot = true;
                    self.phase = GamePhase::Draw;
                } else {
                    // both pots taken
                    // TODO: should game really end? can other team player have valid moves if not used pot?
                    self.phase = GamePhase::Finished(self.winning_team());
                }
            } else {
                // team has already reached one pot, game ends
                self.phase = GamePhase::Finished(self.winning_team());
            }
        } else {
            self.phase = GamePhase::Draw;
        }

        if self.phase == GamePhase::Draw {
            // advance turn
            self.state.player_turn =
                (self.state.player_turn + 1) % self.state.player_team_idxs.len();
            if self.state.player_turn == self.state.first_player {
                self.state.round += 1;
            }
        }

        Ok(())
    }

    pub fn scoreboard(&self) -> Vec<i32> {
        let mut team_scores = Vec::new();
        for team in &self.state().teams {
            // TODO: reached but not used deduction?
            let pot_deduction = if team.has_reached_pot { 0 } else { -100 };

            let runs_score: i32 = team.played_runs.iter().map(|r| r.score()).sum();
            let cards_deduction: i32 = team.players.iter().map(|p| p.hand.value_sum()).sum();

            team_scores.push(pot_deduction + runs_score + cards_deduction);
        }

        team_scores
    }

    pub fn winning_team(&self) -> usize {
        let index_of_max: Option<usize> = self
            .scoreboard()
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(index, _)| index);

        index_of_max.expect("we know there are teams")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawAction {
    DrawOpen,
    DrawPile,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Noop,
}

impl PlayAction {
    pub fn enumerate(
        team_runs: &Vec<Run>,
        player_hand: &Cards,
        moves_allowed: usize,
    ) -> Vec<(PlayAction, i32)> {
        let mut actions = vec![(Noop, 0)];
        // start run sequence actions
        for i in 0..player_hand.len() {
            let card1 = player_hand[i];
            for j in 0..player_hand.len() {
                if i == j {
                    continue;
                };
                let card2 = player_hand[j];

                let card1_is_wildcard = card1.1 == Two || card1.1 == Joker;
                let card2_is_wildcard = card2.1 == Two || card2.1 == Joker;

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
                    if j == k || i == k {
                        continue;
                    };
                    #[cfg(debug_assertions)]
                    eprintln!("p-start");

                    let card3 = player_hand[k];
                    let maybe_run = Run::build_sequence_run(Cards(vec![card1, card2, card3]));
                    if let Ok(run) = maybe_run {
                        #[cfg(debug_assertions)]
                        eprintln!("p-start-ok");
                        let run_score = run.score();
                        actions.push((PlayAction::StartRun(run), run_score));
                    }
                }
            }
        }

        // start run group actions
        for i in 0..player_hand.len() {
            let card1 = player_hand[i];
            for j in (i + 1)..player_hand.len() {
                let card2 = player_hand[j];
                for k in (j + 1)..player_hand.len() {
                    #[cfg(debug_assertions)]
                    eprintln!("p-start-g");

                    let card3 = player_hand[k];

                    let maybe_run = Run::build_group_run(Cards(vec![card1, card2, card3]));
                    if let Ok(run) = maybe_run {
                        #[cfg(debug_assertions)]
                        eprintln!("p-start-g-ok");
                        let run_score = run.score();
                        actions.push((PlayAction::StartRun(run), run_score));
                    }
                }
            }
        }

        // start run group actions

        // append actions
        for i in 0..player_hand.len() {
            let card = Cards(vec![player_hand[i]]);
            for (j, run) in team_runs.iter().enumerate() {
                #[cfg(debug_assertions)]
                eprintln!("p-append");
                if let Ok(new_run) = run.append(&card, Append::Top) {
                    #[cfg(debug_assertions)]
                    eprintln!("p-append-ok");
                    actions.push((
                        PlayAction::AppendTop(j, card.clone()),
                        new_run.score() - run.score(),
                    ));
                }
                if let Ok(new_run) = run.append(&card, Append::Bottom) {
                    #[cfg(debug_assertions)]
                    eprintln!("p-append-ok");
                    actions.push((
                        PlayAction::AppendBottom(j, card.clone()),
                        new_run.score() - run.score(),
                    ));
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
                    if let Ok(new_run) = run.replace_wildcard(k, &card) {
                        #[cfg(debug_assertions)]
                        eprintln!("p-replace-ok");
                        actions.push((
                            PlayAction::ReplaceWildcard(i, k, card),
                            new_run.score() - run.score(),
                        ));
                    }
                }
            }
        }

        // move card actions
        if moves_allowed > 0 {
            for (i, run) in team_runs.iter().enumerate() {
                if run.run_type() == RunType::Group {
                    continue;
                }
                for from in 0..run.cards().len() {
                    let rank_move = run.cards()[from].1;
                    if rank_move != Joker && rank_move != Two && rank_move != Ace {
                        continue;
                    }

                    for to in 0..run.cards().len() {
                        if from == to {
                            continue;
                        }
                        #[cfg(debug_assertions)]
                        eprintln!("p-move");
                        if let Ok(new_run) = run.move_card(from, to) {
                            #[cfg(debug_assertions)]
                            eprintln!("p-move-ok");
                            actions.push((
                                PlayAction::MoveCard(i, from, to),
                                new_run.score() - run.score(),
                            ));
                        }
                    }
                }
            }
        }

        actions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiscardAction(pub Card);

#[cfg(test)]

mod tests {
    use crate::agent::BurracoAgent;
    use super::*;

    // ♣ ♦ ♥ ♠

    #[test]
    fn test_start_run_action() -> Result<(), String> {
        let hand = Cards::of("JK, ♣2, ♣5, ♣7, ♣9, ♣K, ♦6, ♦8, ♦9, ♥10, ♠6, ♠K")?;
        let actions = PlayAction::enumerate(&vec![], &hand, 0);
        for (action, _d_score) in &actions {
            println!("{}", action);
        }

        use std::collections::HashSet;
        let set: HashSet<_> = actions.into_iter().map(|(a, _s)| a).collect();

        let should_enumerate_runs_s = vec![
            "♣5,JK,♣7",
            "♣5,♣2,♣7",
            "♣7,JK,♣9",
            "♣7,♣2,♣9",
            "♦6,♣2,♦8",
            "♦6,JK,♦8",
            "JK,♦8,♦9",
            "♣2,♦8,♦9",
            "♦8,♦9,JK",
            "♦8,♦9,♣2", // TODO group runs
                              // TODO: only accept in certain order to prevent state explosion?
                              // "♦6,♠6,♣2"
        ];

        let should_enumerate_runs: Vec<Result<PlayAction, String>> = should_enumerate_runs_s
            .iter()
            .map(|s| {
                Ok(PlayAction::StartRun(Run::build_sequence_run(Cards::of(
                    s,
                )?)?))
            })
            .collect();

        assert!(should_enumerate_runs.iter().all(|r| r.is_ok()));

        for action in should_enumerate_runs {
            let action = action.unwrap();
            println!("Validating action enumerated: {}", &action);
            assert!(set.contains(&action));
        }

        Ok(())
    }

    #[test]
    fn test_move_action() -> Result<(), String> {
        let hand = Cards::of("♣5")?;
        let actions = PlayAction::enumerate(
            &vec![Run::build_sequence_run(Cards::of("JK,♥3,♥4")?)?],
            &hand,
            1,
        );
        for (action, _d_score) in &actions {
            println!("{}", action);
        }

        use std::collections::HashSet;
        let set: HashSet<_> = actions.into_iter().map(|(a, _s)| a).collect();

        assert_eq!(1, set.len());
        assert!(&set.contains(&PlayAction::Noop));

        Ok(())
    }

    #[test]
    fn test_advance_turn() -> Result<(), String> {
        let mut state = BurracoState::init_with(2, 2);
        // for deterministic test
        state.first_player = 0;
        state.player_turn = 0;

        let mut game = BurracoGame::from(state);
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(0, team);
        assert_eq!(0, player);

        use crate::agent::DumbAgent;
        let mut agent = DumbAgent {};

        // player 1 (T0, P0)
        game.draw(agent.select_draw_action(game.state()))?;
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(0, team);
        assert_eq!(0, player);
        game.play(PlayAction::Noop)?;
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(0, team);
        assert_eq!(0, player);
        game.discard(agent.select_discard_action(&game.current_player().hand, game.state()))?;

        // player 2 (T1, P0)
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(1, team);
        assert_eq!(0, player);

        game.draw(agent.select_draw_action(game.state()))?;
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(1, team);
        assert_eq!(0, player);

        game.play(PlayAction::Noop)?;
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(1, team);
        assert_eq!(0, player);
        game.discard(agent.select_discard_action(&game.current_player().hand, game.state()))?;

        // player 3 (T0, P1)
        let (team, player) = game.state.player_team_idxs[game.state.player_turn];
        assert_eq!(0, team);
        assert_eq!(1, player);

        Ok(())
    }
}
