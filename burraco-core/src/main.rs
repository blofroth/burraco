
use std::io::{Write, stdout};

use burraco::model::BurracoState;

use burraco::actions::BurracoGame;
use burraco::cli_display::print_play_actions;
use burraco::agent::BurracoAgent;
use rand::thread_rng;

fn main() -> Result<(), String> {
    use burraco::actions::PlayAction;
    use burraco::actions::GamePhase::*;

    let state = BurracoState::init_with(2, 2);

    let mut game = BurracoGame::from(state);
    println!("GAME START");
    println!("{}", game);
    println!("---");

    let orig_cards = game.state().cards_total();

    use burraco::agent::ManualCliAgent;
    use burraco::agent::MaxAgent;
    use burraco::agent::DumbAgent;
    use burraco::agent::RandomAgent;
    use burraco::agent::SmartAgent;
    let mut agents: [Box<dyn BurracoAgent>; 4] = [
        Box::new(ManualCliAgent {}),
        // Box::new(MaxAgent {}),
        Box::new(MaxAgent {}),
        Box::new(MaxAgent {}),
        Box::new(SmartAgent {}),
        // Box::new(DumbAgent {}),
        // Box::new(RandomAgent{ rng: thread_rng() })
    ];

    // use agent::DumbAgent;
    // let mut agent = DumbAgent{};
    // use agent::RandomAgent;
    // let mut agent = RandomAgent{ rng: thread_rng() };
    // use agent::ManualCliAgent;
    // let mut agent = ManualCliAgent {};
    

    
    'round: loop {
        if game.state().cards_total() > orig_cards {
            panic!("Cards are procreating! {} vs orig {}", game.state().cards_total(), orig_cards);
        }
        let agent = &mut agents[game.state().player_turn];
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
            print_play_actions(&available_actions, &game.current_team().played_runs);
            let selected_action = agent.select_play_action(available_actions, game.state());
            if let PlayAction::MoveCard(_,_,_) = selected_action {
                moves_allowed -= 1;
            }

            println!("---");
            println!("Agent: {}", agent.display());
            println!("Playing action: {}", selected_action);
            game.play(selected_action)?;
            if let Finished(_) = game.phase() {
                let player_turn = game.state().player_turn;
                let (team, player) = game.state().player_team_idxs[player_turn];
                // TODO: potsetto
                println!("PLAYER WITH EMPTY HAND: team {}, player {} (P{})",  team, player, player_turn);
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
            let player_turn = game.state().player_turn;
                let (team, player) = game.state().player_team_idxs[player_turn];
            println!("PLAYER WITH EMPTY HAND: team {}, player {} (P{})",  team, player, player_turn);
            break 'round;
        }
        println!("---");
        println!("{}", game);
    }

    if let Finished(winning_team) = game.phase() {
        println!("---");
        println!("GAME FINISHED, winner is team: {}", winning_team);
        for (team, score) in game.scoreboard().iter().enumerate() {
            println!("  Team {}: {} points", team, score);
        }
        println!("Winner agents: ");
        for (i, (team, _)) in game.state().player_team_idxs.iter().enumerate() {
            if *team == winning_team {
                println!(" {}", &agents[i].display());
            }
        }
        println!("---");
        println!("{}", game);
    } else {
        println!("undefined game abort");
    }
    stdout().flush().map_err(|e| e.to_string())?;
    Ok( () )
}
