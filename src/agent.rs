use crate::actions::PlayAction;
use crate::actions::DrawAction;
use crate::actions::DiscardAction;
use crate::model::BurracoState;
use crate::model::Cards;

pub trait BurracoAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction;
    fn select_play_action(&mut self, actions: Vec<PlayAction>, state: &BurracoState) -> PlayAction;
    fn select_discard_action(&mut self, hand: &Cards, state: &BurracoState) -> DiscardAction;
}

pub struct DumbAgent {}

impl BurracoAgent for DumbAgent {
    fn select_draw_action(&mut self, state: &BurracoState) -> DrawAction {
        let draw_action = if state.round % 2 == 0 {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        draw_action
    }

    fn select_play_action(&mut self, actions: Vec<PlayAction>, _state: &BurracoState) -> PlayAction {
        actions.into_iter().last().unwrap()
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        DiscardAction(hand[0])
    }
}

use rand::Rng;
use rand::prelude::SliceRandom;
pub struct RandomAgent<R: Rng + ?Sized> {
    pub rng: R
}

impl<R: Rng + ?Sized> BurracoAgent for RandomAgent<R> {

    fn select_draw_action(&mut self, _state: &BurracoState) -> DrawAction {
        let draw_action = if self.rng.gen::<bool>() {
            DrawAction::DrawPile
        } else {
            DrawAction::DrawOpen
        };
        draw_action
    }

    fn select_play_action(&mut self, actions: Vec<PlayAction>, _state: &BurracoState) -> PlayAction {
        actions.choose(&mut self.rng).expect("we know at least noop exists").clone()
    }

    fn select_discard_action(&mut self, hand: &Cards, _state: &BurracoState) -> DiscardAction {
        DiscardAction(*hand.choose(&mut self.rng).expect("game would have ended if empty hand"))
    }

}
