use super::{play::PlayState, GameState};
use crate::player::{AiLevel, ComputerKind, PlayerConf, PlayerKind};
use eframe::{egui, epi};
use reversi_core::Color;

#[derive(Debug, Clone)]
pub(super) struct ConfigState {
    player1: PlayerConf,
    player2: PlayerConf,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            player1: PlayerConf::new("Player 1".into()),
            player2: PlayerConf::new("Player 2".into()),
        }
    }
}

impl ConfigState {
    pub(super) fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut epi::Frame) -> Option<GameState> {
        let Self { player1, player2 } = self;

        let mut new_state = None;

        player_conf(ui, player1);
        player_conf(ui, player2);

        ui.horizontal(|ui| {
            if ui.button("Play").clicked() {
                new_state = Some(GameState::Play(PlayState::new(self.clone())));
            }
            if ui.button("Cancel").clicked() {
                new_state = Some(GameState::Closed);
            }
        });

        new_state
    }

    pub(super) fn player(&self, color: Color) -> &PlayerConf {
        match color {
            Color::Black => &self.player1,
            Color::White => &self.player2,
        }
    }

    pub(super) fn player1(&self) -> &PlayerConf {
        &self.player1
    }

    pub(super) fn player2(&self) -> &PlayerConf {
        &self.player2
    }
}

fn player_conf(ui: &mut egui::Ui, conf: &mut PlayerConf) {
    const PLAYER_KIND: [(PlayerKind, &str); 2] = [
        (PlayerKind::Human, "Human"),
        (PlayerKind::Computer, "Computer"),
    ];
    const COMPUTER_KIND: [(ComputerKind, &str); 5] = [
        (ComputerKind::Random, "Random"),
        (ComputerKind::Ai(AiLevel::Level1), "AI Level1"),
        (ComputerKind::Ai(AiLevel::Level2), "AI Level2"),
        (ComputerKind::Ai(AiLevel::Level3), "AI Level3"),
        (ComputerKind::Ai(AiLevel::Level4), "AI Level4"),
    ];

    ui.heading(&conf.name);
    egui::Grid::new(&conf.name)
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Player Kind");
            for (value, text) in PLAYER_KIND {
                ui.radio_value(&mut conf.player_kind, value, text);
            }
            ui.end_row();

            ui.scope(|ui| {
                ui.set_enabled(conf.player_kind == PlayerKind::Computer);
                ui.label("Computer Kind");
            });
            for (value, text) in COMPUTER_KIND {
                ui.scope(|ui| {
                    ui.set_enabled(conf.player_kind == PlayerKind::Computer);
                    ui.radio_value(&mut conf.computer_kind, value, text);
                });
            }
            ui.end_row();
        });
}
