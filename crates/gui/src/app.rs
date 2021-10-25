use self::game::Game;
use eframe::{egui, epi};

mod game;

#[derive(Debug)]
pub struct App {
    games: Vec<Game>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            games: vec![Default::default()],
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "reversi-gui"
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("New Game").clicked() {
                        self.games.push(Game::default());
                    }

                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        self.games = self
            .games
            .drain(..)
            .filter_map(|mut window| {
                window.ui(ctx, frame);
                window.open().then(|| window)
            })
            .collect();
    }
}
