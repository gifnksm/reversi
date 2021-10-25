use self::{config::ConfigState, play::PlayState};
use eframe::{egui, epi};
use std::sync::atomic::{AtomicU32, Ordering};

mod config;
mod play;

static GAME_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub(super) struct Game {
    title: String,
    state: GameState,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            title: format!("Game #{}", GAME_ID.fetch_add(1, Ordering::Relaxed)),
            state: GameState::Config(Default::default()),
        }
    }
}

impl Game {
    pub(super) fn open(&self) -> bool {
        !matches!(self.state, GameState::Closed)
    }

    pub(super) fn ui(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame) {
        let Self { title, state } = self;

        let mut open = true;
        egui::Window::new(title)
            .open(&mut open)
            .auto_sized()
            .show(ctx, |ui| state.ui(ui, frame));
        if !open {
            *state = GameState::Closed;
        }
    }
}

#[derive(Debug)]
enum GameState {
    Config(ConfigState),
    Play(PlayState),
    Closed,
}

impl GameState {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame) {
        let new_state = match self {
            GameState::Config(state) => state.ui(ui, frame),
            GameState::Play(state) => state.ui(ui, frame),
            GameState::Closed => None,
        };
        if let Some(new_state) = new_state {
            *self = new_state;
        }
    }
}
