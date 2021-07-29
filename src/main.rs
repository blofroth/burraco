use std::ops::Deref;
use std::ops::DerefMut;

use Suit::*;
use Rank::*;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    Jokers
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            Clubs => "♣",
            Diamonds => "♦", 
            Hearts => "♥",
            Spades => "♠",
            Jokers => " "
        };
        write!(f, "{}", s)
    }
}

static SUITS: [Suit; 4] = [
    Clubs,
    Diamonds,
    Hearts,
    Spades
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match &self {
            Two   => " 2",
            Three => " 3",
            Four  => " 4",
            Five  => " 5",
            Six   => " 6",
            Seven => " 7",
            Eight => " 8",
            Nine  => " 9",
            Ten   => "10",
            Jack  => " J",
            Queen => " Q",
            King  => " K",
            Ace   => " A",
            Joker => "JK"
        };
        write!(f, "{}", s)
    }
}

static SUIT_RANK: [Rank; 13] = [
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
];

#[derive(Debug, Clone, Copy)]
struct Card(Suit, Rank);

#[derive(Debug, Clone)]
struct Cards(Vec<Card>);

impl Deref for Cards {
    type Target = Vec<Card>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cards {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.0, self.1)
    }
}

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for card in self.iter() {
            write!(f, "{}", card)?;
        }
        write!(f, "]")
    }
}


fn build_deck(num_jokers: usize) -> Vec<Card> {
    let mut deck = Vec::new();
    
    for suit in SUITS.iter() {
        for rank in SUIT_RANK.iter() {
            deck.push( Card(*suit, *rank) );
        }
    }

    for _i in 0..num_jokers {
        deck.push(Card(Jokers,Joker))
    }

    deck
}

fn shuffle(cards: &mut Vec<Card>) {
    use rand::prelude::*;
    let mut rng = thread_rng();

    cards.shuffle(&mut rng)   
}

#[derive(Debug, Clone)]
struct Player {
    hand: Cards
}


#[derive(Debug, Clone)]
struct Team {
    players: Vec<Player>,
    played_runs: Vec<Cards>,
    has_reached_pot: bool,
    has_used_pot: bool,
}
#[derive(Debug, Clone)]
struct BurracoGame {
    num_teams: usize,
    num_team_players: usize,
    draw_pile: Cards,
    open_pile: Cards,
    pot1: Cards,
    pot2: Cards,
    teams: Vec<Team>,
    player_turn: (usize, usize), // team idx, teamplayer idx
    phase: GamePhase
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GamePhase {
    Draw, 
    Play, 
    Discard,
    Finished
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrawAction {
    DrawOpen,
    DrawPile
}

#[derive(Debug, Clone)]
enum Run {
    SequenceRun(Cards),
    GroupRun(Cards)
}

#[derive(Debug, Clone)]
enum PlayAction {
    StartRun(Cards),
    AppendTop(usize, Cards), // run_idx
    AppendBottom(usize, Cards), // run_idx
    ReplaceWildcard(usize, usize), // run_idx
    MoveWildcard(usize, usize, usize), // run_idx, from, to
    Noop
}

#[derive(Debug, Clone, Copy)]
struct DiscardAction(Card);

impl fmt::Display for BurracoGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Burraco table: \n")?;
        for t in 0..self.num_teams {
            write!(f, "  Team {}\n", t)?;
            for p in 0..self.num_team_players {
                write!(f, "    Player {}-{}: {} cards \n", t, p, self.teams[t].players[p].hand.len())?;
            }
            write!(f, "    Runs played\n")?;
            for run in &self.teams[t].played_runs {
                write!(f, "    - {}\n", run)?;
            }
        }
        write!(f, "  Draw pile: {} cards \n", self.draw_pile.len() )?;
        write!(f, "  Open pile: {} \n", self.open_pile )?;
        write!(f, "\n")?;
        write!(f, "  Pot 1: {} cards \n", self.pot1.len() )?;
        write!(f, "  Pot 2: {} cards \n", self.pot2.len() )?;
        let (team, player) = self.player_turn;
        write!(f, "Current turn: team {} player {} \n", team, player)?;
        write!(f, "Current phase: {:?} \n", &self.phase )?;
        write!(f, "---")
    }
}


impl BurracoGame {

    fn init_with(num_teams: usize, num_team_players: usize) -> BurracoGame {
        // 2 decks
        let mut deck = build_deck(3);
        deck.append(&mut build_deck(3));
    
        shuffle(&mut deck);
    
        let pot1 = Cards(deck.split_off(deck.len() - 11));
        let pot2 = Cards(deck.split_off(deck.len() - 11));
    
        let mut teams = Vec::new();
        for _i in 0..num_teams {
            let mut team_players = Vec::new();
            for _j in 0..num_team_players {
                team_players.push(Player {
                    hand: Cards(deck.split_off(deck.len() - 11))
                });
            }
            teams.push(Team { 
                players: team_players,
                has_reached_pot: false,
                has_used_pot: false,
                played_runs: Vec::new()
            })
        }
        let open_pile = Cards(deck.split_off(deck.len() - 1));
        let draw_pile = Cards(deck);
        BurracoGame {
            num_teams,
            num_team_players,
            draw_pile,
            open_pile,
            pot1,
            pot2,
            teams,
            player_turn: (0,0), // TODO: randomize,
            phase: GamePhase::Draw
        }
    }

    fn draw(&mut self, action: DrawAction) -> Result<(),String> {
        dbg!(&action);

        let (team, player) = self.player_turn;
        

        if self.phase != GamePhase::Draw {
            return Err(format!("Draw action invalid when phase is: {:?}", self.phase));
        }

        match action {
            DrawAction::DrawOpen => {
                let open_pile = &mut self.open_pile;
                self.teams[team].players[player].hand.append(open_pile);

            },
            DrawAction::DrawPile => {
                let draw_pile_len = self.draw_pile.len();
                let draw_pile = &mut self.draw_pile;
                self.teams[team].players[player].hand.append(&mut draw_pile.split_off(draw_pile_len - 1));
            }
        }

        self.phase = GamePhase::Play;
        Ok( () )
    }
    
    fn play(&mut self, action: PlayAction) -> Result<(),String> {
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
    
    fn discard(&mut self, action: DiscardAction) -> Result<(),String>  {
        dbg!(&action);
        if self.phase != GamePhase::Discard {
            return Err(format!("Discard action invalid when phase is: {:?}", self.phase));
        }
        
        self.phase = GamePhase::Draw;
        let (mut team, mut player) = self.player_turn;
        let next_team = (team + 1) %  self.num_teams;
        if next_team == 0 && (team + 1) * (player + 1) == self.num_teams * self.num_team_players {
            player = player + 1;
        }
        team = next_team;
        self.player_turn = (team, player);
        Ok( () )
    }

    fn current_player(&self) -> &Player {
        let (team, player) = self.player_turn;
        &self.teams[team].players[player]
    }
}


fn main() -> Result<(), String> {

    let mut game = BurracoGame::init_with(2, 2);

    println!("{}", game);
    game.draw( DrawAction::DrawPile)?;
    println!("{}", game);
    game.play( PlayAction::Noop)?;
    println!("{}", game);
    game.discard(DiscardAction(game.current_player().hand[0]))?;
    println!("{}", game);

    Ok( () )
}
