use super::{play::PlayState, GameState};
use crate::player::{AiLevel, ComputerKind, PlayerConf, PlayerKind};
use eframe::egui;

#[derive(Debug, Clone, Default)]
pub(super) struct ConfigState {
    player1: PlayerConf,
    player2: PlayerConf,
}

impl ConfigState {
    pub(super) fn ui(&mut self, ui: &mut egui::Ui) -> Option<GameState> {
        let Self { player1, player2 } = self;

        let mut new_state = None;

        player_conf(ui, "Player 1", player1);
        player_conf(ui, "Player 2", player2);

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

    pub(super) fn player1(&self) -> &PlayerConf {
        &self.player1
    }

    pub(super) fn player2(&self) -> &PlayerConf {
        &self.player2
    }
}

fn player_conf(ui: &mut egui::Ui, name: &str, conf: &mut PlayerConf) {
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

    ui.heading(name);
    egui::Grid::new(name)
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
