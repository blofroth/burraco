
use model::BurracoState;

use actions::BurracoGame;
use cli_display::print_play_actions;
use agent::BurracoAgent;
use rand::thread_rng;

mod model;
mod actions;
mod cli_display;
mod agent;

fn main() -> Result<(), String> {
    use actions::PlayAction;
    use actions::GamePhase::*;

    let state = BurracoState::init_with(2, 2);

    let mut game = BurracoGame::from(state);
    println!("GAME START");
    println!("{}", game);
    println!("---");

    let orig_cards = game.state().cards_total();

    // use agent::DumbAgent;
    // let mut agent = DumbAgent{};
    use agent::RandomAgent;
    let mut agent = RandomAgent{ rng: thread_rng() };
    //use agent::ManualCliAgent;
    //let mut agent = ManualCliAgent {};
    
    'round: loop {
        if game.state().cards_total() > orig_cards {
            panic!("Cards are procreating! {} vs orig {}", game.state().cards_total(), orig_cards);
        }
        // poor mans randomization :)
        let draw_action = agent.select_draw_action(game.state());

        println!("Agent: {}", agent.display());
        println!("Draw action: {}", &draw_action);
        game.draw(draw_action)?;
        if let Finished(_) = game.phase() {
            println!("OUT OF PILE CARDS!");
            break 'round;
        }
        println!("---");
        println!("{}", game);
        
        // probably enough even if new runs are created?
        let mut moves_allowed = game.current_team().played_runs.len();

        // play until noop
        'player_plays: loop {
            if game.state().cards_total() > orig_cards {
                panic!("Cards are procreating! {} vs orig {}", game.state().cards_total(), orig_cards);
            }

            let available_actions = PlayAction::enumerate(&game.current_team().played_runs, &game.current_player().hand, moves_allowed);
            print_play_actions(&available_actions);
            let selected_action = agent.select_play_action(available_actions, game.state());
            if let PlayAction::MoveCard(_,_,_) = selected_action {
                moves_allowed -= 1;
            }

            println!("---");
            println!("Agent: {}", agent.display());
            println!("Playing action: {}", selected_action);
            game.play(selected_action)?;
            if let Finished(_) = game.phase() {
                let (team, player) = game.state().player_turn;
                // TODO: potsetto
                println!("PLAYER WITH EMPTY HAND: team {}, player {}",  team, player);
                break 'round;
            }
            println!("---");
            println!("{}", game);
            if game.phase() != Play {
                break 'player_plays;
            }
        }

        let discard_action = agent.select_discard_action(&game.current_player().hand, game.state());
        println!("Agent: {}", agent.display());
        println!("Discard action: {}", discard_action);
        game.discard(discard_action)?;
        if let Finished(_) = game.phase() {
            let (team, player) = game.state().player_turn;
            println!("PLAYER WITH EMPTY HAND: team {}, player {}",  team, player);
            break 'round;
        }
        println!("---");
        println!("{}", game);
    }

    if let Finished(winning_team) = game.phase() {
        println!("---");
        println!("GAME FINISHED, winner is team: {}", winning_team);
        println!("---");
        println!("{}", game);
    } else {
        println!("undefined game abort");
    }

    Ok( () )
}
