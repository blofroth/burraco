use std::ops::Deref;
use std::ops::DerefMut;
use std::str;

use Rank::*;
use Suit::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    Jokers,
}

pub static SUITS: [Suit; 4] = [Clubs, Diamonds, Hearts, Spades];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Two, // Pinella
    Numerical(i16),
    Jack,
    Queen,
    King,
    Ace,
    Joker,
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

impl Rank {
    pub fn index(&self) -> i16 {
        match self {
            Two => 2,
            Numerical(num) => *num,
            Jack => 11,
            Queen => 12,
            King => 13,
            Ace => 14,
            Joker => -2, // not in any sequence
        }
    }

    pub fn value(&self) -> i32 {
        match self.index() {
            3..=7 => 5,
            8..=13 => 10,
            14 => 15,
            2 => 20,
            -2 => 30,
            _ => panic!("invalid Rank index value"),
        }
    }

    pub fn from_index(index: i16) -> Rank {
        match index {
            2 => Two,
            num @ 3..=10 => Numerical(num),
            11 => Jack,
            12 => Queen,
            13 => King,
            14 => Ace,                                // assume 1 is handled outside
            _ => panic!("unsupported index mapping"), // assume we don't need to handle joker here
        }
    }
    pub fn next(&self) -> Rank {
        match self {
            Ace => Two,
            Joker => panic!("no defined next rank for Joker"),
            any => Rank::from_index(any.index() + 1),
        }
    }

    pub fn prev(&self) -> Option<Rank> {
        match self {
            Ace => None,
            Joker => None,
            any => Some(Rank::from_index(any.index() - 1)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card(pub Suit, pub Rank);

impl Card {
    fn parse(string: &str) -> Result<Card, String> {
        if string == "JK" {
            return Ok(Card(Jokers, Joker));
        }
        if string.len() < 3 {
            return Err("Invalid suit".into());
        }
        // unicode for these are 3 bytes
        let suit = match &string[0..3] {
            "♣" => Clubs,
            "♦" => Diamonds,
            "♥" => Hearts,
            "♠" => Spades,
            _ => return Err("Unknown suit character".into()),
        };
        let rank = match &string[3..] {
            "2" => Two,
            num @ ("3" | "4" | "5" | "6" | "7" | "8" | "9" | "10") => {
                Numerical(num.parse::<i16>().map_err(|e| e.to_string())?)
            }
            "J" => Jack,
            "Q" => Queen,
            "K" => King,
            "A" => Ace,
            _ => return Err("Unknown rank".into()),
        };
        Ok(Card(suit, rank))
    }

    /// for use with sorting to canonicalize card sequences
    pub fn val_tpl(&self) -> (i16, i16) {
        let suit_val = match self.0 {
            Clubs => 1,
            Diamonds => 2,
            Hearts => 3,
            Spades => 4,
            Jokers => 0,
        };

        let rank_val = self.1.index();

        (suit_val, rank_val)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cards(pub Vec<Card>);

impl Cards {
    fn build_deck(num_jokers: usize) -> Cards {
        let mut deck = Vec::new();

        for suit in SUITS.iter() {
            for rank in SUIT_RANK.iter() {
                deck.push(Card(*suit, *rank));
            }
        }

        for _i in 0..num_jokers {
            deck.push(Card(Jokers, Joker))
        }

        Cards(deck)
    }

    pub fn of(expr: &str) -> Result<Cards, String> {
        if expr.trim().is_empty() {
            return Ok(Cards(vec![]));
        }
        let cards: Result<Vec<_>, _> = expr
            .split(',')
            .map(|part| Card::parse(part.trim()))
            .collect();
        Ok(Cards(cards?))
    }

    pub fn drain_back(&mut self, num_cards: usize) -> Cards {
        let index = self.len() - num_cards;
        Cards(self.split_off(index))
    }

    pub fn sort(&mut self) {
        self.sort_by(|a, b| Card::val_tpl(a).cmp(&Card::val_tpl(b)))
    }

    pub fn value_sum(&self) -> i32 {
        self.iter().map(|c| c.1.value()).sum()
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
    pub hand: Cards,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub players: Vec<Player>,
    pub played_runs: Vec<Run>,
    pub has_reached_pot: bool,
    pub has_used_pot: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunType {
    Sequence,
    Group,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Run {
    cards: Cards,
    run_type: RunType,
}

// ensure append only?
impl Run {
    pub fn cards(&self) -> &Cards {
        &self.cards
    }

    pub fn run_type(&self) -> RunType {
        self.run_type
    }

    pub fn is_burraco(&self) -> bool {
        self.cards.len() >= 7
    }

    pub fn burraco_value(&self) -> i32 {
        if self.is_burraco() {
            match self.run_type {
                RunType::Sequence => {
                    let mut num_clean_in_sequence = 0;
                    let mut max_num_clean_in_sequence = 0;

                    let mut last_was_clean = false;

                    for i in 1..self.cards.len() {
                        let prev_card = self.cards[i - 1];
                        let card = self.cards[i];

                        let mut clean_transition = false;

                        if prev_card.0 == card.0 {
                            if prev_card.1 == Two {
                                if card.1 == Numerical(3) {
                                    // Two used cleanly as two in sequence
                                    clean_transition = true
                                }
                            } else {
                                clean_transition = true;
                            }
                        }

                        if i == self.cards.len() - 1 {
                            // last card

                            if card.1 == Two {
                                // cover dirty case not validated by above prev clean two check
                                clean_transition = false;
                            }

                            if last_was_clean && clean_transition {
                                // count up for this card
                                num_clean_in_sequence += 1;
                            }
                        }

                        if clean_transition {
                            // count up for prev card
                            num_clean_in_sequence += 1;
                        } else {
                            if last_was_clean {
                                num_clean_in_sequence += 1; // max should include this card
                            }
                            max_num_clean_in_sequence =
                                max_num_clean_in_sequence.max(num_clean_in_sequence);
                            num_clean_in_sequence = 0;
                        }

                        last_was_clean = clean_transition;
                    }
                    if num_clean_in_sequence == self.cards.len() {
                        if num_clean_in_sequence == 13 {
                            300
                        } else {
                            200
                        }
                    } else if max_num_clean_in_sequence >= 7
                        && max_num_clean_in_sequence == self.cards.len() - 1
                    {
                        // has to be a wildcard in either end
                        150
                    } else {
                        100
                    }
                }
                RunType::Group => {
                    100 // TODO
                }
            }
        } else {
            0
        }
    }

    /// to (burraco_score, cards_score)
    pub fn score(&self) -> i32 {
        let (burraco_score, cards_score) = self.score_burraco_cards();
        burraco_score + cards_score
    }
    pub fn score_burraco_cards(&self) -> (i32, i32) {
        let cards_score: i32 = self.cards.value_sum();

        let burraco_score = self.burraco_value();
        // TODO: burraco_score
        (burraco_score, cards_score)
    }

    pub fn build_sequence_run(cards: Cards) -> Result<Run, String> {
        if cards.len() < 3 {
            return Err("Need at least 3 cards to create a sequence run".into());
        }
        let first_known_suit = cards
            .iter().find(|c| c.1 != Joker && c.1 != Two)
            .ok_or_else(|| "Need at least some non wild cards for sequence run".to_string())?
            .0;
        let num_same_two = cards
            .iter()
            .filter(|c| c.0 == first_known_suit && c.1 == Rank::Two)
            .count();
        let num_other_two = cards
            .iter()
            .filter(|c| c.0 != first_known_suit && c.1 == Rank::Two)
            .count();
        let num_joker = cards.iter().filter(|c| c.1 == Rank::Joker).count();

        // two Two:s of main suit could be allowed, cap to 1 in this count
        // NOTE: we assume two decks
        if num_same_two.min(1) + num_other_two + num_joker > 1 {
            return Err("Too many wildcards in sequence run".into());
        }

        let mut wildcard_replaces = None;

        for i in 0..cards.len() {
            let prev_card = if i > 0 { Some(cards[i - 1]) } else { None };
            let card = cards[i];
            let next_card = if i < cards.len() - 1 {
                Some(cards[i + 1])
            } else {
                None
            };

            let curr_wildcard_replacement = match (prev_card, card, next_card) {
                // Two used as two, in same suit (REF1)
                (_, Card(suit1, Two), Some(Card(suit2, Numerical(3))))
                    if suit1 == first_known_suit && suit2 == first_known_suit =>
                {
                    None
                }
                // Special case for same suit Two as Ace
                (None, Card(suit1, Two), Some(Card(suit2, Two)))
                    if suit1 == first_known_suit && suit2 == first_known_suit =>
                {
                    Some((i, Ace))
                }
                // Two used as two with wildcard as three
                (_, Card(suit, Two), Some(Card(_, Two | Joker))) if suit == first_known_suit => {
                    None
                }
                // Two used as wildcard, in any other suit
                (Some(Card(_, prev_rank)), Card(_, Joker | Two), _) => Some((i, prev_rank.next())),
                (None, Card(_, Joker | Two), Some(Card(_, next_rank)))
                    if next_rank.prev().is_some() =>
                {
                    Some((i, next_rank.prev().unwrap()))
                }
                _ => None,
            };

            if curr_wildcard_replacement.is_some() {
                if wildcard_replaces.is_some() {
                    return Err("You have already used a wildcard in this run".into());
                } else {
                    wildcard_replaces = curr_wildcard_replacement;
                }
                let valid_wildcard_sequence = 
                    !matches!((prev_card, card.1, next_card), (Some(Card(_, Ace)), _, None));

                if !valid_wildcard_sequence {
                    return Err("Cannot extend from Ace with wildcard".into());
                }
            } else {
                if card.0 != first_known_suit {
                    return Err("Mismatched suit in sequence run".into());
                }

                if let Some(prev) = prev_card {
                    let prev_rank = match wildcard_replaces {
                        Some((prev_i, rank)) if prev_i == i - 1 => rank,
                        _ => prev.1,
                    };

                    let valid_rank_sequence = match (prev_rank, card.1, next_card) {
                        (Ace, curr, _) => curr == Two,
                        _ => prev_rank.index() + 1 == card.1.index(),
                    };

                    if !valid_rank_sequence {
                        return Err(format!("Invalid sequence: {} to {}", prev, &card));
                    }
                }
            }
        }

        Ok(Run {
            cards,
            run_type: RunType::Sequence,
        })
    }

    pub fn build_group_run(cards: Cards) -> Result<Run, String> {
        if cards.len() < 3 {
            return Err("Need at least 3 cards to create a group run".into());
        }
        let mut cards = cards;
        cards.sort_by_key(|c| (c.1 == Two, c.val_tpl())); // canonicalize format

        let first_known_rank = cards
            .iter()
            .filter(|c| c.1 != Joker && c.1 != Two)
            .map(|c| c.1)
            .next()
            .unwrap_or(Two);

        let mut num_wildcards_used = 0;
        for i in 0..cards.len() {
            let card = cards[i];

            if card.1 != first_known_rank && card.1 != Two && card.1 != Joker {
                return Err(format!(
                    "Mismatched rank in group sequence (pos {}): {}, expected {}",
                    i, card.1, first_known_rank
                ));
            }

            if card.1 == Joker || (card.1 == Two && first_known_rank != Two) {
                // wildcard used

                if num_wildcards_used > 0 {
                    return Err(format!(
                        "Too many wildcards in group run (pos {}): {}, already used {}",
                        i, card, num_wildcards_used
                    ));
                }

                num_wildcards_used += 1;
            }
        }

        Ok(Run {
            run_type: RunType::Group,
            cards,
        })
    }

    pub fn append(&self, cards: &Cards, append_to: Append) -> Result<Run, String> {
        let mut new_cards = self.cards.clone();
        match append_to {
            Append::Top => {
                let end = new_cards.len();
                new_cards.splice(end..end, cards.iter().cloned());
            }
            Append::Bottom => {
                new_cards.splice(0..0, cards.iter().cloned());
            }
        };

        match self.run_type {
            RunType::Sequence => Run::build_sequence_run(new_cards),
            RunType::Group => Run::build_group_run(new_cards),
        }
    }

    pub fn replace_wildcard(&self, at: usize, card: &Card) -> Result<Run, String> {
        use std::mem;

        let mut new_cards = self.cards().clone();
        if at >= new_cards.len() {
            return Err("Cannot replace at invalid position".into());
        }

        let old_card = mem::replace(&mut new_cards[at], *card);
        new_cards.insert(0, old_card);
        let new_run = match self.run_type() {
            RunType::Sequence => Run::build_sequence_run(new_cards)?,
            // we sort these, so we should still be able to get 150
            RunType::Group => {
                return Err("No point in replacing wildcard in group, use append".into())
            }
        };
        Ok(new_run)
    }

    pub fn move_card(&self, from: usize, to: usize) -> Result<Run, String> {
        let mut new_cards = self.cards().clone();
        if from >= new_cards.len() || to >= new_cards.len() || from == to || to == from + 1 {
            return Err("Cannot move from/to invalid position".into());
        }

        let wildcard = new_cards[from];
        // assume we don't need to verify card types here
        // should only be wildcards and ace that are allowed to move

        dbg!(&from);
        dbg!(&to);
        new_cards.insert(to, wildcard);
        dbg!(&new_cards);
        let remove_idx = if to < from { from + 1 } else { from };
        new_cards.remove(remove_idx);
        dbg!(&new_cards);

        let new_run = match self.run_type() {
            RunType::Sequence => Run::build_sequence_run(new_cards)?,
            RunType::Group => return Err("No point in moving card in group".into()),
        };
        Ok(new_run)
    }
}

pub enum Append {
    Top,
    Bottom,
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
    pub player_turn: usize,
    pub first_player: usize,
    /// (team_idx, in_team_idx)
    pub player_team_idxs: Vec<(usize, usize)>,
    pub round: u32,
}

impl BurracoState {
    pub fn curr_team_player(&self) -> (usize, usize) {
        self.player_team_idxs[self.player_turn]
    }

    pub fn curr_team(&self) -> usize {
        let (team, _) = self.player_team_idxs[self.player_turn];
        team
    }

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
                    hand: deck.drain_back(11),
                });
                team_players[j].hand.sort();
            }
            teams.push(Team {
                players: team_players,
                has_reached_pot: false,
                has_used_pot: false,
                played_runs: Vec::new(),
            })
        }

        let mut player_team_idxs = Vec::new();
        for p in 0..num_team_players {
            for t in 0..num_teams {
                player_team_idxs.push((t, p))
            }
        }

        let starting_player = thread_rng().gen_range(0..player_team_idxs.len());

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
            player_turn: starting_player,
            first_player: starting_player,
            player_team_idxs,
            round: 0,
        }
    }

    // for sanity checking
    pub fn cards_total(&self) -> usize {
        let team_cards: usize = self
            .teams
            .iter()
            .map(|t| {
                let run_sum: usize = t.played_runs.iter().map(|r| r.cards.len()).sum();
                let hand_sum: usize = t.players.iter().map(|p| p.hand.len()).sum();
                run_sum + hand_sum
            })
            .sum();

        let pile_cards =
            self.draw_pile.len() + self.open_pile.len() + self.pot1.len() + self.pot2.len();
        team_cards + pile_cards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ♣ ♦ ♥ ♠

    #[test]
    fn test_build_empty() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_build_simple() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣3,♣4,♣5")?)).is_ok());
        Ok(())
    }

    #[test]
    fn test_too_many_twos() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣A,♣2,♣2,♣3")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_wildcard_one_ok() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣2,♣2,♣3")?)).is_ok());
        Ok(())
    }

    #[test]
    fn test_normal_two_ok() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣A,♣2,♣3")?)).is_ok());
        Ok(())
    }

    #[test]
    fn test_wildcard_three_ok() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣A,♣2,♣2")?)).is_ok());
        Ok(())
    }

    #[test]
    fn test_bad_suit() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣3,♦4,♣5")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_ok_joker() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣3,JK,♣5")?)).is_ok());
        Ok(())
    }

    #[test]
    fn test_too_many_wildcards() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣3,JK,JK,♣6")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_bad_non_wildcard_two() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♣2,♣4,♣2,♣6")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_bad_final_wildcard() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♥K,♥A,JK")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_bad_mid_wildcard() -> Result<(), String> {
        assert!(dbg!(Run::build_sequence_run(Cards::of("♥3,JK,♥4")?)).is_err());
        Ok(())
    }

    #[test]
    fn test_move_seq_wildcard() -> Result<(), String> {
        let orig_run = Run::build_sequence_run(Cards::of("JK,♥3,♥4")?)?;
        assert!(dbg!(orig_run.move_card(0, 1)).is_err());
        Ok(())
    }

    #[test]
    fn test_burraco_score() -> Result<(), String> {
        assert_eq!(
            0,
            Run::build_sequence_run(Cards::of("♠3,♠4,♠5")?)
                .unwrap()
                .burraco_value()
        );

        assert_eq!(
            100,
            Run::build_sequence_run(Cards::of("JK,♠4,♠5,♠6,♠7,♠8,♠9")?)
                .unwrap()
                .burraco_value()
        );
        assert_eq!(
            100,
            Run::build_sequence_run(Cards::of("♠3,♠4,♠5,♠6,♠7,♠8,♠2")?)
                .unwrap()
                .burraco_value()
        );
        assert_eq!(
            100,
            Run::build_sequence_run(Cards::of("♠2,♠3,♠4,♠2,♠6,♠7,♠8")?)
                .unwrap()
                .burraco_value()
        );

        assert_eq!(
            150,
            Run::build_sequence_run(Cards::of("♠3,♠4,♠5,♠6,♠7,♠8,♠9,JK")?)
                .unwrap()
                .burraco_value()
        );
        assert_eq!(
            150,
            Run::build_sequence_run(Cards::of("♠2,♠3,♠4,♠5,♠6,♠7,♠8,♠2")?)
                .unwrap()
                .burraco_value()
        );

        assert_eq!(
            200,
            Run::build_sequence_run(Cards::of("♠3,♠4,♠5,♠6,♠7,♠8,♠9")?)
                .unwrap()
                .burraco_value()
        );

        assert_eq!(
            300,
            Run::build_sequence_run(Cards::of("♠2,♠3,♠4,♠5,♠6,♠7,♠8,♠9,♠10,♠J,♠Q,♠K,♠A")?)
                .unwrap()
                .burraco_value()
        );
        assert_eq!(
            300,
            Run::build_sequence_run(Cards::of("♠A,♠2,♠3,♠4,♠5,♠6,♠7,♠8,♠9,♠10,♠J,♠Q,♠K")?)
                .unwrap()
                .burraco_value()
        );

        Ok(())
    }
}
