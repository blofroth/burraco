use std::ops::Deref;
use std::ops::DerefMut;


use Suit::*;
use Rank::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    Jokers
}

pub static SUITS: [Suit; 4] = [
    Clubs,
    Diamonds,
    Hearts,
    Spades
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    Two, // Pinella
    Numerical(i16),
    Jack,
    Queen,
    King,
    Ace,
    Joker
}

pub static SUIT_RANK: [Rank; 13] = [
    Two,
    Numerical(3),
    Numerical(4),
    Numerical(5),
    Numerical(6),
    Numerical(7),
    Numerical(8),
    Numerical(9),
    Numerical(10),
    Jack,
    Queen,
    King,
    Ace,
];

#[derive(Debug, Clone, Copy)]
pub struct Card(pub Suit, pub Rank);

#[derive(Debug, Clone)]
pub struct Cards(pub Vec<Card>);

impl Cards {
    fn build_deck(num_jokers: usize) -> Cards {
        let mut deck = Vec::new();
        
        for suit in SUITS.iter() {
            for rank in SUIT_RANK.iter() {
                deck.push( Card(*suit, *rank) );
            }
        }
    
        for _i in 0..num_jokers {
            deck.push(Card(Jokers,Joker))
        }
    
        Cards(deck)
    }

    pub fn drain_back(&mut self, num_cards: usize) -> Cards {
        let index = self.len() - num_cards;
        Cards(self.split_off(index))
    }
    
    pub fn sort(&mut self) {
        fn val_tpl(card: &Card) -> (i16, i16) {
            let suit_val = match card.0 {
                Clubs => 1,
                Diamonds => 2,
                Hearts => 3,
                Spades => 4,
                Jokers => 0
            };

            let rank_val = match card.1 {
                Two => 2,
                Numerical(num) => num,
                Jack => 11,
                Queen => 12,
                King => 13,
                Ace => 14,
                Joker => 0,
            };

            (suit_val, rank_val)
        }
        self.sort_by(|a, b| {
            val_tpl(a).cmp(&val_tpl(b))
        })
    } 
}

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

#[derive(Debug, Clone)]
pub struct Player {
    pub hand: Cards
}

#[derive(Debug, Clone)]
pub struct Team {
    pub players: Vec<Player>,
    pub played_runs: Vec<Run>,
    pub has_reached_pot: bool,
    pub has_used_pot: bool,
}

#[derive(Debug, Clone)]
pub enum Run {
    SequenceRun(Cards),
    GroupRun(Cards)
}

#[derive(Debug, Clone)]
pub struct BurracoState {
    pub num_teams: usize,
    pub num_team_players: usize,
    pub draw_pile: Cards,
    pub open_pile: Cards,
    pub pot1: Cards,
    pub pot2: Cards,
    pub teams: Vec<Team>,
    pub player_turn: (usize, usize), // team idx, teamplayer idx
}

impl BurracoState {

    pub fn init_with(num_teams: usize, num_team_players: usize) -> BurracoState {
        use rand::prelude::*;
        // 2 decks
        let mut deck = Cards::build_deck(3);
        deck.append(&mut Cards::build_deck(3));
    
        deck.shuffle(&mut thread_rng());
    
        let pot1 = deck.drain_back(11);
        let pot2 = deck.drain_back(11);
    
        let mut teams = Vec::new();
        for _i in 0..num_teams {
            let mut team_players = Vec::new();
            for j in 0..num_team_players {
                team_players.push(Player {
                    hand: deck.drain_back(11)
                });
                team_players[j].hand.sort();
            }
            teams.push(Team { 
                players: team_players,
                has_reached_pot: false,
                has_used_pot: false,
                played_runs: Vec::new()
            })
        }
        let open_pile = deck.drain_back(1);
        let draw_pile = deck;
        BurracoState {
            num_teams,
            num_team_players,
            draw_pile,
            open_pile,
            pot1,
            pot2,
            teams,
            player_turn: (0,0), // TODO: randomize,
        }
    }

    pub fn current_player(&self) -> &Player {
        let (team, player) = self.player_turn;
        &self.teams[team].players[player]
    }
}