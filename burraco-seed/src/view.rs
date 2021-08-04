// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use crate::RootMsg;
use burraco::actions::GamePhase;
use burraco::model::BurracoState;
use burraco::model::Card;
use burraco::model::Cards;
use burraco::model::Rank;
use burraco::model::Run;
use burraco::model::Suit;
use seed::prelude::web_sys::Event;
use seed::{prelude::*, *};

use super::InitMsg;
use super::*;
use Msg::*;

// ------ ------
//     View
// ------ ------

fn draw_action_buttons(actions: &[String]) -> Node<RootMsg> {
    let buttons: Vec<_> = actions
        .iter()
        .enumerate()
        .map(|(action_i, s)| {
            li![button![
                s,
                ev(Ev::Click, move |_| RootMsg::Game(Draw(action_i))),
            ]]
        })
        .collect();
    ul![buttons]
}

fn play_action_buttons(actions: &[(usize, String)]) -> Node<RootMsg> {
    let actions = actions.to_owned();
    ul![actions.into_iter().map(|(action_i, s)| {
        li![button![
            s,
            ev(Ev::Click, move |_| RootMsg::Game(Play(action_i))),
        ]]
    })]
}

fn discard_action_buttons(actions: &[String]) -> Node<RootMsg> {
    let buttons: Vec<_> = actions
        .iter()
        .enumerate()
        .map(|(_action_i, s)| li![button![s, ev(Ev::Click, move |_| RootMsg::Game(Discard)),]])
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

fn card(card: &Card) -> Node<RootMsg> {
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
    let (suit_text, suit_class) = card_css_suit_class(card);
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

fn clickable_card<MsU: 'static>(
    card: &Card,
    handler: impl FnOnce(Event) -> MsU + 'static + Clone,
) -> Node<RootMsg>
where
    MsU: 'static,
{
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
    let (suit_text, suit_class) = card_css_suit_class(card);
    div![
        C![
            "card",
            IF!(card.1 != Rank::Joker => format!("rank-{}", rank_lower)),
            suit_class,
        ],
        span![C!["rank"], rank_upper],
        span![C!["suit"], suit_text],
        ev(Ev::Click, handler)
    ]
}

fn deck(cards: &Cards) -> Node<RootMsg> {
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

fn view_hidden_hands(state: &BurracoState) -> Vec<Node<RootMsg>> {
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

fn hidden_hand(cards: &Cards) -> Node<RootMsg> {
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

fn hand(cards: &Cards, selected_cards: &HashSet<usize>) -> Node<RootMsg> {
    /*
    <ul class="hand">
        <li>
            <div class="card ...">...</div>
        </li>
    </ul> */
    let card_nodes: Vec<_> = cards.iter().enumerate().map(
        |(i, c)| {
            li![
                IF!( selected_cards.contains(&i) => strong!(clickable_card(c, move |_| RootMsg::Game(Select(i))))),
                IF!( !selected_cards.contains(&i) => clickable_card(c, move |_| RootMsg::Game(Select(i)))),
            ]
        }
    ).collect();
    div![
        ul![C!["hand"], card_nodes],
        // format!("selected: {:?}", selected_cards)
    ]
}

fn open_cards(cards: &Cards) -> Node<RootMsg> {
    /*
    <ul class="table">
        <li>
            <div class="card ...">...</div>
        </li>
    </ul> */
    let card_nodes: Vec<_> = cards.iter().map(|c| li![card(c)]).collect();
    ul![C!["table"], card_nodes]
}

fn runs(runs: &[Run]) -> Node<RootMsg> {
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

fn view_credits(_model: &Model) -> Vec<Node<RootMsg>> {
    raw![include_str!("../credits.html")]
}

pub fn view_game(maybe_model: &Option<GameModel>) -> Node<RootMsg> {
    // For some reason IF macro does not like sending in unwrapped
    if let Some(model) = maybe_model {
        let other_team_runs: Vec<_> = model.game.state().teams[1..]
            .iter()
            .map(|t| runs(&t.played_runs))
            .collect();

        div![
            div!["Current player turn: ", model.game.state().player_turn],
            div!["Last move: ", &model.last_move],
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
                    div![
                        "Your hand (Player 0, Team 0)",
                        hand(
                            &model.game.state().teams[0].players[0].hand,
                            &model.selected_cards
                        )
                    ],
                    IF!(!model.is_manual_turn() => button!["Advance other players", ev(Ev::Click, |_| RootMsg::Game(Advance)),]),
                    IF!(model.is_manual_turn() && model.game.phase() == GamePhase::Draw => div!["Draw action choices", draw_action_buttons(&model.draw_choices)]),
                    IF!(model.is_manual_turn() && model.game.phase() == GamePhase::Play => div!["Play action choices (select cards for more)", play_action_buttons(&model.play_choices)]),
                    IF!(model.is_manual_turn() && model.game.phase() == GamePhase::Discard => div!["Discard action choices", discard_action_buttons(&model.discard_choices)]),
                ],
            ],
        ]
    } else {
        div![] // should not happen
    }
}

pub fn view_init(model: &Model) -> Node<RootMsg> {
    div![
        h2!("Start a new game"),
        div![
            p!["Number of teams: ", model.init_options.num_teams,],
            p![input![
                attrs! {
                    At::AutoFocus => AtValue::None,
                    At::Value => model.init_options.num_teams ,
                    "type" => "range",
                    "min" => 2,
                    "max" => 3
                },
                input_ev(Ev::Input, move |value| RootMsg::Init(InitMsg::SetNumTeams(
                    value.parse().unwrap()
                )))
            ]],
            p![
                "Number players per team: ",
                model.init_options.num_team_players
            ],
            p![input![
                attrs! {
                    At::AutoFocus => AtValue::None,
                    At::Value => model.init_options.num_team_players ,
                    "type" => "range",
                    "min" => 1,
                    "max" => 2 // TODO: reevaluate?
                },
                input_ev(Ev::Input, move |value| RootMsg::Init(InitMsg::SetPlayers(
                    value.parse().unwrap()
                )))
            ]],
            p![
                "Enable manual player: ",
                input![
                    attrs! {
                        At::AutoFocus => AtValue::None,
                        At::Type => "checkbox",
                        At::Checked => model.init_options.enable_manual_player.as_at_value(),
                        At::AutoFocus => AtValue::None
                    },
                    input_ev(Ev::Input, move |_value| RootMsg::Init(InitMsg::FlipManual))
                ]
            ],
            p!["Automatic player logic: "],
            model.init_options.agents.iter().enumerate().map(|(i, a)| {
                let is_manual = model.init_options.enable_manual_player && i == 0;
                let agent_type_buttons: Vec<Node<RootMsg>> = [
                    AgentType::Dumb,
                    AgentType::Smart,
                    AgentType::Max,
                    AgentType::Random,
                ]
                .iter()
                .map(|t| {
                    button![
                        format!("Use {:?}", t),
                        ev(Ev::Click, move |_| RootMsg::Init(InitMsg::SetAgent(i, *t))),
                    ]
                })
                .collect();
                div![
                    p![format!(
                        "Player {}: {:?}",
                        i,
                        if is_manual { AgentType::Manual } else { *a }
                    )],
                    IF!( !is_manual => p![agent_type_buttons] )
                ]
            }),
            button![
                "Create game",
                ev(Ev::Click, move |_| RootMsg::Init(InitMsg::Create)),
            ]
        ]
    ]
}

// `view` describes what to display.
pub fn view(model: &Model) -> Node<RootMsg> {
    div![
        h1!("Burraco"),
        div!["Error messages", pre![model.error_msg.to_string()]],
        IF!(model.game_model.is_some() => view_game(&model.game_model)),
        IF!(model.game_model.is_none() => view_init(model)),
        view_credits(model)
    ]
}
