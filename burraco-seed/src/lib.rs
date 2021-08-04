// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use burraco::actions::BurracoGame;
use burraco::actions::DiscardAction;
use burraco::actions::DrawAction;
use burraco::actions::GamePhase;
use burraco::actions::PlayAction;
use burraco::agent::BurracoAgent;
use burraco::agent::DumbAgent;
use burraco::agent::ManualCliAgent;
use burraco::agent::SmartAgent;
use burraco::model::BurracoState;
use burraco::model::Card;
use burraco::model::Cards;
use burraco::model::Rank;
use burraco::model::Run;
use burraco::model::Suit;
use seed::{prelude::*, *};

// ------ ------
//     Init
// ------ ------

// set to out of range for no manual players
//const MANUAL_PLAYER: usize = 0;
const MANUAL_PLAYER: usize = 999;

// `init` describes what should happen when your app started.
fn init(_: Url, _orders: &mut impl Orders<Msg>) -> Model {
    let state = BurracoState::init_with(2, 2);

    let game = BurracoGame::from(state);

    let mut model = Model {
        game,
        agents: vec![
            Box::new(SmartAgent {}), // we will detect this special case as manual player
            Box::new(SmartAgent {}),
            Box::new(SmartAgent {}),
            Box::new(SmartAgent {}),
        ],
        last_move: "".into(),
        draw_choices: vec![],
        play_choices: vec![],
        discard_choices: vec![],
        error_msg: "".into(),
        curr_player_moves_allowed: 0,
    };
    model.update_choices();

    model
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    game: BurracoGame,
    agents: Vec<Box<dyn BurracoAgent>>,
    last_move: String,
    draw_choices: Vec<String>,
    play_choices: Vec<String>,
    discard_choices: Vec<String>,
    curr_player_moves_allowed: usize,
    error_msg: String,
}

impl Model {
    fn update_choices(&mut self) {
        if self.game.state().player_turn == MANUAL_PLAYER {
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
}

// ------ ------
//    Update
// ------ ------

// (Remove the line below once any of your `Msg` variants doesn't implement `Copy`.)
#[derive(Clone, Debug)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    Draw(usize),
    Play(usize),
    Discard(usize),
    Advance,
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    if model.game.state().player_turn == MANUAL_PLAYER {
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
            (p, m) => model
                .error_msg
                .push_str(&mut format!("Bad action ({:?}) in phase: {:?}", m, p)),
        }
    } else {
        if let Msg::Advance = msg {
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
                    let curr_move = model.agents[model.game.state().player_turn]
                        .select_play_action(
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
                        .select_discard_action(
                            &model.game.current_player().hand,
                            model.game.state(),
                        );
                    model.last_move =
                        format!("{} - Player {}", &curr_move, model.game.state().player_turn);
                    model.game.discard(curr_move).expect("valid discard");
                    model.curr_player_moves_allowed = 0;
                }
                GamePhase::Finished(winning_team) => {
                    (model.last_move = format!(
                        "Winner is team {}. Scores: {:?}",
                        winning_team,
                        &model.game.scoreboard()
                    ))
                }
            }
        }
    }
    model.update_choices();
}

// ------ ------
//     View
// ------ ------

fn draw_action_buttons(actions: &Vec<String>) -> Node<Msg> {
    let buttons: Vec<_> = actions
        .iter()
        .enumerate()
        .map(|(action_i, s)| li![button![s, ev(Ev::Click, move |_| Msg::Draw(action_i)),]])
        .collect();
    ul![buttons]
}

fn play_action_buttons(actions: &Vec<String>) -> Node<Msg> {
    let buttons: Vec<_> = actions
        .iter()
        .enumerate()
        .map(|(action_i, s)| li![button![s, ev(Ev::Click, move |_| Msg::Play(action_i)),]])
        .collect();
    ul![buttons]
}

fn discard_action_buttons(actions: &Vec<String>) -> Node<Msg> {
    let buttons: Vec<_> = actions
        .iter()
        .enumerate()
        .map(|(action_i, s)| li![button![s, ev(Ev::Click, move |_| Msg::Discard(action_i)),]])
        .collect();
    ul![buttons]
}

fn card_css_suit_class(card: &Card) -> (String, String) {
    let text = if Suit::Jokers == card.0 {
        "Joker".into()
    } else {
        format!("{}", card.0)
    };
    let class = match card.0 {
        Suit::Clubs => "clubs",
        Suit::Diamonds => "diams",
        Suit::Hearts => "hearts",
        Suit::Spades => "spades",
        Suit::Jokers => "joker",
    };
    (text, class.into())
}

fn card(card: &Card) -> Node<Msg> {
    /*
       <div class="card rank-j clubs">
       <span class="rank">J</span>
       <span class="suit">♣</span>
       </div>
    */
    let rank_upper = if card.1 == Rank::Joker {
        "-".to_string()
    } else {
        format!("{}", card.1)
    };
    let rank_lower = rank_upper.to_lowercase();
    let (suit_text, suit_class) = card_css_suit_class(&card);
    div![
        C![
            "card",
            IF!(card.1 != Rank::Joker => format!("rank-{}", rank_lower)),
            suit_class
        ],
        span![C!["rank"], rank_upper],
        span![C!["suit"], suit_text]
    ]
}

fn deck(cards: &Cards) -> Node<Msg> {
    let range = 0..(cards.len().min(32)); // some css lib limitation
                                          /*
                                          <ul class="deck">
                                              <li>
                                                  <div class="card back">*</div>
                                              </li>
                                          </ul> */
    ul![
        C!["deck"],
        range
            .into_iter()
            .map(|_i| { li![div![C!["card", "back"], "*"]] })
    ]
}

fn view_hidden_hands(state: &BurracoState) -> Vec<Node<Msg>> {
    let mut hands = Vec::new();
    for i in 1..state.player_team_idxs.len() {
        let curr_player = state.player_turn;
        let curr_i = (curr_player + i) % state.player_team_idxs.len();
        let (team, player) = state.player_team_idxs[curr_i];

        let playing = IF!(curr_i == state.player_turn => b!("[Playing]"));
        hands.push(p![format!("Player {} (Team {})", i, team), playing]);
        hands.push(hidden_hand(&state.teams[team].players[player].hand));
    }
    hands
}

fn hidden_hand(cards: &Cards) -> Node<Msg> {
    let range = 0..(cards.len().min(32)); // some css lib limitation
                                          /*
                                          <ul class="deck">
                                              <li>
                                                  <div class="card back">*</div>
                                              </li>
                                          </ul> */
    ul![
        C!["hand"],
        range
            .into_iter()
            .map(|_i| { li![div![C!["card", "back"], "*"]] })
    ]
}

fn hand(cards: &Cards) -> Node<Msg> {
    /*
    <ul class="hand">
        <li>
            <div class="card ...">...</div>
        </li>
    </ul> */
    let card_nodes: Vec<_> = cards.iter().map(|c| li![card(c)]).collect();
    ul![C!["hand"], card_nodes]
}

fn open_cards(cards: &Cards) -> Node<Msg> {
    /*
    <ul class="table">
        <li>
            <div class="card ...">...</div>
        </li>
    </ul> */
    let card_nodes: Vec<_> = cards.iter().map(|c| li![card(c)]).collect();
    ul![C!["table"], card_nodes]
}

fn runs(runs: &Vec<Run>) -> Node<Msg> {
    /*
    <ul class="table">
        <li>
            <div class="card ...">...</div>
        </li>
    </ul> */
    let run_nodes: Vec<_> = runs
        .iter()
        .map(|r| {
            div![
                open_cards(r.cards()),
                format!("({:?}, {} p)", r.run_type(), r.score())
            ]
        })
        .collect();
    div![run_nodes]
}

// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    let other_team_runs: Vec<_> = model.game.state().teams[1..]
        .iter()
        .map(|t| runs(&t.played_runs))
        .collect();
    div![
        h1!("Burraco"),
        //pre![format!("{}", model.game)],
        //pre![format!("{}", model.last_view)],
        div!["Last move: ", &model.last_move],
        div!["Error messages", pre![format!("{}", model.error_msg)]],
        div![
            C![
                "playingCards",
                "fourColours",
                "rotateHand",
                "simpleCards",
                "inText"
            ],
            style! {
                St::Display => "flex",
                St::Padding => px(10)
            },
            div![
                style! {
                    "flex" => "1"
                },
                "Other team runs",
                other_team_runs
            ],
            div![
                style! {
                    "flex" => "1"
                },
                "Your team runs",
                runs(&model.game.state().teams[0].played_runs)
            ],
        ],
        div![
            C!["playingCards", "fourColours", "rotateHand", "simpleCards"],
            style! {
                St::Display => "flex",
                St::Padding => px(10)
            },
            div![
                style! {
                    "flex" => "1"
                },
                "Piles",
                open_cards(&model.game.state().open_pile),
                deck(&model.game.state().draw_pile)
            ],
            div![
                style! {
                    "flex" => "1"
                },
                "Pots",
                deck(&model.game.state().pot1),
                deck(&model.game.state().pot2)
            ],
            div![
                style! {
                    "flex" => "1"
                },
                "Other hands",
                view_hidden_hands(model.game.state())
            ],
            div![
                style! {
                    "flex" => "1"
                },
                IF!(model.game.state().player_turn != MANUAL_PLAYER => button!["Advance other players", ev(Ev::Click, |_| Msg::Advance),]),
                IF!(model.game.state().player_turn == MANUAL_PLAYER && model.game.phase() == GamePhase::Draw => div!["Draw action choices", draw_action_buttons(&model.draw_choices)]),
                IF!(model.game.state().player_turn == MANUAL_PLAYER && model.game.phase() == GamePhase::Play => div!["Play action choices", play_action_buttons(&model.play_choices)]),
                IF!(model.game.state().player_turn == MANUAL_PLAYER && model.game.phase() == GamePhase::Discard => div!["Discard action choices", discard_action_buttons(&model.discard_choices)]),
                div![
                    "Your hand (Player 0, Team 0)",
                    hand(&model.game.state().teams[0].players[0].hand)
                ],
            ],
        ],
        view_credits(model)
    ]
}

fn view_credits(_model: &Model) -> Vec<Node<Msg>> {
    raw![include_str!("../credits.html")]
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
