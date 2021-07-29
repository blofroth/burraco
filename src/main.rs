
use model::BurracoState;

use actions::BurracoGame;

mod model;
mod actions;
mod cli_display;

fn main() -> Result<(), String> {
    use actions::DrawAction;
    use actions::PlayAction;
    use actions::DiscardAction;

    let state = BurracoState::init_with(2, 2);

    let mut game = BurracoGame::from(state);

    println!("{}", game);
    game.draw( DrawAction::DrawPile)?;
    println!("{}", game);
    game.play( PlayAction::Noop)?;
    println!("{}", game);
    game.discard(DiscardAction(game.state.current_player().hand[0]))?;
    println!("{}", game);

    Ok( () )
}
