
use model::BurracoState;

use actions::BurracoGame;
use cli_display::print_play_actions;

mod model;
mod actions;
mod cli_display;

fn main() -> Result<(), String> {
    use actions::DrawAction;
    use actions::PlayAction;
    use actions::DiscardAction;
    use actions::GamePhase::*;

    let state = BurracoState::init_with(2, 2);

    let mut game = BurracoGame::from(state);
    println!("GAME START");
    println!("{}", game);
    println!("---");
    
    loop {
        // poor mans randomization :)
        let draw_action = if game.state().round % 2 == 0 {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        println!("Draw action: {}", &draw_action);
        game.draw(draw_action)?;
        if let Finished(_) = game.phase() {
            println!("OUT OF PILE CARDS!");
            break;
        }
        println!("---");
        println!("{}", game);
        let available_actions = PlayAction::enumerate(&game.current_team().played_runs, &game.current_player().hand);
        print_play_actions(&available_actions);
        let selected_action = available_actions.into_iter().last().unwrap();
        println!("Playing action: {}", selected_action);
        game.play( selected_action)?;
        if let Finished(_) = game.phase() {
            let (team, player) = game.state().player_turn;
            println!("PLAYER WITH EMPTY HAND: team {}, player {}",  team, player);
            break;
        }
        println!("---");
        println!("{}", game);
        let discard_action = DiscardAction(game.current_player().hand[0]);
        println!("Discard action: {}", discard_action);
        game.discard(discard_action)?;
        if let Finished(_) = game.phase() {
            let (team, player) = game.state().player_turn;
            println!("PLAYER WITH EMPTY HAND: team {}, player {}",  team, player);
            break;
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
