// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use burraco::actions::BurracoGame;
use burraco::actions::DiscardAction;
use burraco::actions::DrawAction;
use burraco::actions::GamePhase;
use burraco::actions::PlayAction;
use burraco::agent::*;
use burraco::model::BurracoState;
use seed::prelude::*;

mod view;

// ------ ------
//     Init
// ------ ------

/// if manual player is used, it has this player number
const MANUAL_PLAYER: usize = 0;

// `init` describes what should happen when your app started.
fn init(_: Url, _orders: &mut impl Orders<RootMsg>) -> Model {
    // log!(format!("game model is some: {}", model.game_model.is_some()));

    Model {
        game_model: None,
        init_options: InitOptions {
            num_teams: 2,
            num_team_players: 2,
            enable_manual_player: true,
            agents: vec![
                AgentType::Smart,
                AgentType::Smart,
                AgentType::Smart,
                AgentType::Smart,
            ],
        },
        error_msg: "".into(),
    }
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
pub struct Model {
    game_model: Option<GameModel>,
    init_options: InitOptions,
    error_msg: String,
}

pub struct GameModel {
    game: BurracoGame,
    enable_manual_player: bool,
    agents: Vec<Box<dyn BurracoAgent>>,
    last_move: String,
    draw_choices: Vec<String>,
    play_choices: Vec<String>,
    discard_choices: Vec<String>,
    curr_player_moves_allowed: usize,
}

impl GameModel {
    fn update_choices(&mut self) {
        if self.is_manual_turn() {
            match self.game.phase() {
                GamePhase::Draw => {
                    self.draw_choices.append(&mut vec![
                        format!("{}", DrawAction::DrawOpen),
                        format!("{}", DrawAction::DrawPile),
                    ]);

                    self.play_choices.clear();
                    self.discard_choices.clear();
                }

                GamePhase::Play => {
                    self.play_choices.clear();
                    let actions = PlayAction::enumerate(
                        &self.game.current_team().played_runs,
                        &self.game.current_player().hand,
                        1,
                    );

                    let mut action_strs: Vec<_> = actions
                        .into_iter()
                        .map(|(a, _d_score)| format!("{}", a))
                        .collect();
                    self.play_choices.append(&mut action_strs);

                    self.draw_choices.clear();
                    self.discard_choices.clear();
                }

                GamePhase::Discard => {
                    let cards = self.game.current_player().hand.0.clone();
                    let mut card_str: Vec<_> = cards.iter().map(|c| format!("{}", c)).collect();

                    self.discard_choices.append(&mut card_str);

                    self.draw_choices.clear();
                    self.play_choices.clear();
                }
                _ => (),
            }
        } else {
            self.draw_choices.clear();
            self.play_choices.clear();
            self.discard_choices.clear();
        }
    }

    pub fn is_manual_turn(&self) -> bool {
        self.enable_manual_player && self.game.state().player_turn == MANUAL_PLAYER
    }
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Clone, Debug)]
// `Msg` describes the different events you can modify state with.
pub enum RootMsg {
    Game(Msg),
    Init(InitMsg),
}

#[derive(Copy, Clone, Debug)]
pub enum AgentType {
    Dumb,
    Smart,
    Random,
    Max,
    Manual,
}

#[derive(Clone, Debug)]
pub struct InitOptions {
    pub num_teams: usize,
    pub num_team_players: usize,
    pub enable_manual_player: bool,
    pub agents: Vec<AgentType>,
}

#[derive(Clone, Debug)]
pub enum InitMsg {
    Create,
    SetNumTeams(usize),
    SetPlayers(usize),
    SetAgent(usize, AgentType),
    FlipManual,
}

#[derive(Clone, Debug)]
pub enum Msg {
    Draw(usize),
    Play(usize),
    Discard(usize),
    Advance,
}

fn update(msg: RootMsg, model: &mut Model, orders: &mut impl Orders<RootMsg>) {
    if let Some(game_model) = &mut model.game_model {
        if let RootMsg::Game(game_message) = msg {
            let res = update_game(game_message, game_model, orders);
            if let Err(s) = res {
                model.error_msg.push_str(&s);
            }
        }
    } else {
        match msg {
            RootMsg::Init(InitMsg::Create) => {
                let state = BurracoState::init_with(
                    model.init_options.num_teams,
                    model.init_options.num_team_players,
                );

                let game = BurracoGame::from(state);

                fn create_agent(agent_type: AgentType) -> Box<dyn BurracoAgent> {
                    let agent: Box<dyn BurracoAgent> = match agent_type {
                        AgentType::Dumb => Box::new(DumbAgent {}),
                        AgentType::Smart => Box::new(SmartAgent {}),
                        AgentType::Max => Box::new(MaxAgent {}),
                        AgentType::Random => Box::new(random_agent_thread_rng()),
                        _ => unimplemented!(),
                    };
                    agent
                }

                let mut game_model = GameModel {
                    game,
                    enable_manual_player: model.init_options.enable_manual_player,
                    agents: model
                        .init_options
                        .agents
                        .iter()
                        .map(|a| create_agent(*a))
                        .collect(),
                    last_move: "".into(),
                    draw_choices: vec![],
                    play_choices: vec![],
                    discard_choices: vec![],
                    curr_player_moves_allowed: 0,
                };

                game_model.update_choices();
                model.game_model.replace(game_model);
            }
            RootMsg::Init(InitMsg::SetNumTeams(num)) => model.init_options.num_teams = num,
            RootMsg::Init(InitMsg::SetPlayers(num)) => model.init_options.num_team_players = num,
            RootMsg::Init(InitMsg::FlipManual) => {
                model.init_options.enable_manual_player = !model.init_options.enable_manual_player
            }
            RootMsg::Init(InitMsg::SetAgent(idx, agent_type)) => {
                model.init_options.agents[idx] = agent_type
            }
            _ => model
                .error_msg
                .push_str("Bad message when no game initated"),
        }
    }
}

// `update` describes how to handle each `Msg`.
fn update_game(
    msg: Msg,
    model: &mut GameModel,
    _: &mut impl Orders<RootMsg>,
) -> Result<(), String> {
    if model.is_manual_turn() {
        match (model.game.phase(), msg) {
            (GamePhase::Draw, Msg::Draw(idx)) => {
                let curr_move = [DrawAction::DrawOpen, DrawAction::DrawPile][idx];
                model.last_move =
                    format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                model.game.draw(curr_move).expect("valid draw action")
            }
            (GamePhase::Play, Msg::Play(idx)) => {
                let actions = PlayAction::enumerate(
                    &model.game.current_team().played_runs,
                    &model.game.current_player().hand,
                    7,
                );
                let curr_move = actions[idx].0.clone();
                model.last_move =
                    format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                model.game.play(curr_move).expect("valid play action")
            }
            (GamePhase::Discard, Msg::Discard(idx)) => {
                let curr_move = DiscardAction(model.game.current_player().hand[idx]);
                model.last_move =
                    format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                model.game.discard(curr_move).expect("valid discard")
            }
            (p, m) => return Err(format!("Bad action ({:?}) in phase: {:?}", m, p)),
        }
    } else if let Msg::Advance = msg {
        match model.game.phase() {
            GamePhase::Draw => {
                let curr_move = model.agents[model.game.state().player_turn]
                    .select_draw_action(model.game.state());
                model.last_move =
                    format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                model.game.draw(curr_move).expect("valid draw action");
                model.curr_player_moves_allowed = model.game.current_team().played_runs.len();
            }
            GamePhase::Play => {
                let curr_move = model.agents[model.game.state().player_turn].select_play_action(
                    PlayAction::enumerate(
                        &model.game.current_team().played_runs,
                        &model.game.current_player().hand,
                        7,
                    ),
                    model.game.state(),
                );
                model.last_move =
                    format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                model.game.play(curr_move).expect("valid play action")
            }
            GamePhase::Discard => {
                let curr_move = model.agents[model.game.state().player_turn]
                    .select_discard_action(&model.game.current_player().hand, model.game.state());
                model.last_move =
                    format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                model.game.discard(curr_move).expect("valid discard");
                model.curr_player_moves_allowed = 0;
            }
            GamePhase::Finished(winning_team) => {
                model.last_move = format!(
                    "Winner is team {}. Scores: {:?}",
                    winning_team,
                    &model.game.scoreboard()
                )
            }
        }
    }
    model.update_choices();
    Ok(())
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view::view);
}
