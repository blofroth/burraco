
use crate::model::BurracoState;
use crate::model::Card;
use crate::model::Cards;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Draw, 
    Play, 
    Discard,
    Finished(usize) // winning team
}

pub struct BurracoGame {
    pub state: BurracoState,
    pub phase: GamePhase
}

impl BurracoGame {
    pub fn from(state: BurracoState) -> BurracoGame {
        BurracoGame {
            state,
            phase: GamePhase::Draw
        }
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
        self.phase = GamePhase::Play;
        Ok( () )
    }
    
    pub fn play(&mut self, action: PlayAction) -> Result<(),String> {
        dbg!(&action);
        if self.phase != GamePhase::Play {
            return Err(format!("Play action invalid when phase is: {:?}", self.phase));
        }

        match action {
            PlayAction::Noop => {}
            _ => return Err(format!("Not implemented for {:?}", &action))
        }

        self.phase = GamePhase::Discard;
        Ok( () )
    }
    
    pub fn discard(&mut self, action: DiscardAction) -> Result<(),String>  {
        dbg!(&action);
        if self.phase != GamePhase::Discard {
            return Err(format!("Discard action invalid when phase is: {:?}", self.phase));
        }
        
        self.phase = GamePhase::Draw;
        let (mut team, mut player) = self.state.player_turn;
        let next_team = (team + 1) %  self.state.num_teams;
        if next_team == 0 && (team + 1) * (player + 1) == self.state.num_teams * self.state.num_team_players {
            player = player + 1;
        }
        team = next_team;
        self.state.player_turn = (team, player);
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
    StartRun(Cards),
    AppendTop(usize, Cards), // run_idx
    AppendBottom(usize, Cards), // run_idx
    ReplaceWildcard(usize, usize), // run_idx
    MoveWildcard(usize, usize, usize), // run_idx, from, to
    Noop
}

#[derive(Debug, Clone, Copy)]
pub struct DiscardAction(pub Card);