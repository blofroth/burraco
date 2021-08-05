use burraco::actions::Action;
use burraco::actions::BurracoGame;
use burraco::actions::GamePhase::*;
use burraco::actions::PlayAction;
use burraco::agent::create_agent;
use burraco::agent::AgentType;
use burraco::model::BurracoState;

type PlayedAction = (u32, usize, Action);

fn run_match(teams: &[&[AgentType]], seed: u64) -> Result<(usize, Vec<PlayedAction>), String> {
    let num_teams = teams.len();
    let num_team_players = teams[0].len();
    let num_players = num_teams * num_team_players;

    let mut agents = vec![];
    for i in 0..num_players {
        let team = i % num_teams;
        let team_player = i / num_teams;
        let agent_type = &teams[team][team_player];
        let agent = create_agent(*agent_type);
        agents.push(agent);
    }

    let state = BurracoState::init_seeded(num_teams, num_team_players, Some(seed));
    let mut game = BurracoGame::from(state);

    let orig_cards = game.state().cards_total();

    let mut played_actions: Vec<PlayedAction> = Vec::new();

    'round: loop {
        let player = game.state().player_turn;
        let round = game.state().round;
        if game.state().cards_total() > orig_cards {
            panic!(
                "Cards are procreating! {} vs orig {}",
                game.state().cards_total(),
                orig_cards
            );
        }
        let agent = &mut agents[game.state().player_turn];
        let draw_action = agent.select_draw_action(game.state());
        played_actions.push((round, player, Action::Draw(draw_action)));

        game.draw(draw_action)?;
        if let Finished(_) = game.phase() {
            break 'round;
        }
        // probably enough even if new runs are created?
        let mut moves_allowed = game.current_team().played_runs.len();

        // play until noop
        'player_plays: loop {
            if game.state().cards_total() > orig_cards {
                panic!(
                    "Cards are procreating! {} vs orig {}",
                    game.state().cards_total(),
                    orig_cards
                );
            }

            let available_actions = PlayAction::enumerate(
                &game.current_team().played_runs,
                &game.current_player().hand,
                moves_allowed,
            );
            let selected_action = agent.select_play_action(available_actions, game.state());
            played_actions.push((round, player, Action::Play(selected_action.clone())));
            if let PlayAction::MoveCard(_, _, _) = selected_action {
                moves_allowed -= 1;
            }

            game.play(selected_action)?;
            if let Finished(_) = game.phase() {
                break 'round;
            }
            if game.phase() != Play {
                break 'player_plays;
            }
        }

        let discard_action = agent.select_discard_action(&game.current_player().hand, game.state());
        played_actions.push((round, player, Action::Discard(discard_action)));
        game.discard(discard_action)?;
        if let Finished(_) = game.phase() {
            break 'round;
        }
    }

    if let Finished(winning_team) = game.phase() {
        Ok((winning_team, played_actions))
    } else {
        panic!(
            "Game should not finish in other state than Finished, was {:?}",
            game.phase()
        );
    }
}

#[test]
fn run_dumb_vs_smart() -> Result<(), String> {
    let team_agents = [
        &[AgentType::Dumb, AgentType::Dumb][..],
        &[AgentType::Smart, AgentType::Smart][..],
    ];

    const NUM_ROUNDS: usize = 100;
    let mut team_wins = vec![0; team_agents.len()];

    for i in 0..NUM_ROUNDS {
        let (winner, _actions) = run_match(&team_agents[..], i as u64)?;
        team_wins[winner] += 1;
    }

    println!("Played {} rounds: {:?}", NUM_ROUNDS, team_agents);
    for (i, team_win) in team_wins.iter().enumerate() {
        println!(
            " - Team {} ({:?}) - {} wins ({} %)",
            i,
            team_agents[i],
            team_win,
            (*team_win as f64) * 100f64 / NUM_ROUNDS as f64
        );
    }

    // CHANGE DETECTOR test
    assert_eq!(31, team_wins[0]); // dumb % vs smart
    Ok(())
}

#[test]
fn run_smart_vs_max() -> Result<(), String> {
    let team_agents = [
        &[AgentType::Smart, AgentType::Smart][..],
        &[AgentType::Max, AgentType::Max][..],
    ];

    const NUM_ROUNDS: usize = 100;
    let mut team_wins = vec![0; team_agents.len()];

    for i in 0..NUM_ROUNDS {
        let (winner, _actions) = run_match(&team_agents[..], i as u64)?;
        team_wins[winner] += 1;
    }

    println!("Played {} rounds: {:?}", NUM_ROUNDS, team_agents);
    for (i, team_win) in team_wins.iter().enumerate() {
        println!(
            " - Team {} ({:?}) - {} wins ({} %)",
            i,
            team_agents[i],
            team_win,
            (*team_win as f64) * 100f64 / NUM_ROUNDS as f64
        );
    }

    // CHANGE DETECTOR test
    assert_eq!(61, team_wins[0]); // smart % vs max
    Ok(())
}

#[test]
fn run_smartmax_vs_max() -> Result<(), String> {
    let team_agents = [
        &[AgentType::Smart, AgentType::Max][..],
        &[AgentType::Max, AgentType::Max][..],
    ];

    const NUM_ROUNDS: usize = 100;
    let mut team_wins = vec![0; team_agents.len()];

    for i in 0..NUM_ROUNDS {
        let (winner, _actions) = run_match(&team_agents[..], i as u64)?;
        team_wins[winner] += 1;
    }

    println!("Played {} rounds: {:?}", NUM_ROUNDS, team_agents);
    for (i, team_win) in team_wins.iter().enumerate() {
        println!(
            " - Team {} ({:?}) - {} wins ({} %)",
            i,
            team_agents[i],
            team_win,
            (*team_win as f64) * 100f64 / NUM_ROUNDS as f64
        );
    }

    // CHANGE DETECTOR test
    assert_eq!(65, team_wins[0]); // smart max % vs max
    Ok(())
}

#[test]
fn run_smartmax_vs_smart() -> Result<(), String> {
    let team_agents = [
        &[AgentType::Smart, AgentType::Max][..],
        &[AgentType::Smart, AgentType::Smart][..],
    ];

    const NUM_ROUNDS: usize = 100;
    let mut team_wins = vec![0; team_agents.len()];

    for i in 0..NUM_ROUNDS {
        let (winner, _actions) = run_match(&team_agents[..], i as u64)?;
        team_wins[winner] += 1;
    }

    println!("Played {} rounds: {:?}", NUM_ROUNDS, team_agents);
    for (i, team_win) in team_wins.iter().enumerate() {
        println!(
            " - Team {} ({:?}) - {} wins ({} %)",
            i,
            team_agents[i],
            team_win,
            (*team_win as f64) * 100f64 / NUM_ROUNDS as f64
        );
    }

    // CHANGE DETECTOR test
    assert_eq!(52, team_wins[0]); // smart max % vs smart
    Ok(())
}

#[test]
fn run_random_vs_max() -> Result<(), String> {
    let team_agents = [
        &[AgentType::SeededRandom(0), AgentType::SeededRandom(0)][..],
        &[AgentType::Max, AgentType::Max][..],
    ];

    const NUM_ROUNDS: usize = 100;
    let mut team_wins = vec![0; team_agents.len()];

    for i in 0..NUM_ROUNDS {
        let (winner, _actions) = run_match(&team_agents[..], i as u64)?;
        team_wins[winner] += 1;
    }

    println!("Played {} rounds: {:?}", NUM_ROUNDS, team_agents);
    for (i, team_win) in team_wins.iter().enumerate() {
        println!(
            " - Team {} ({:?}) - {} wins ({} %)",
            i,
            team_agents[i],
            team_win,
            (*team_win as f64) * 100f64 / NUM_ROUNDS as f64
        );
    }

    // CHANGE DETECTOR test
    assert_eq!(41, team_wins[0]); // smart % vs max
    Ok(())
}
