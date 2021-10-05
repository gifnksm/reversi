use super::config::ConfigState;
use crate::player::{AiLevel, ComputerKind, PlayerConf, PlayerKind};
use eframe::egui;
use rand::prelude::*;
use reversi_com::{Com, NextMove, WeightEvaluator};
use reversi_core::{Color, Game, Pos};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{mpsc, Arc},
    thread,
};

mod board;

#[derive(Debug)]
pub(super) struct PlayState {
    config: ConfigState,
    computer1: Option<Computer>,
    computer2: Option<Computer>,
    game: Game,
    last_put: Option<Pos>,
    messages: Vec<String>,
    state: GameState,
}

#[derive(Debug)]
enum GameState {
    Init,
    WaitHuman,
    WaitComputer(mpsc::Receiver<NextMove>),
    GameOver,
}

#[derive(Debug)]
enum Computer {
    Ai(Arc<Com>, Arc<WeightEvaluator>),
    Random,
}

impl Computer {
    fn from_config(config: &PlayerConf) -> Option<Computer> {
        if config.player_kind != PlayerKind::Computer {
            return None;
        }

        let ai_level = match config.computer_kind {
            ComputerKind::Random => return Some(Computer::Random),
            ComputerKind::Ai(ai_level) => ai_level,
        };

        let com = match ai_level {
            AiLevel::Level1 => Com::new(2, 8, 10),
            AiLevel::Level2 => Com::new(4, 10, 12),
            AiLevel::Level3 => Com::new(6, 12, 14),
            AiLevel::Level4 => Com::new(8, 14, 16),
        };

        // TODO: error handling
        let evaluator = || -> Result<WeightEvaluator, Box<dyn std::error::Error>> {
            let data_path = Path::new("dat").join("evaluator.dat");
            if data_path.exists() {
                let file = File::open(data_path)?;
                let buf = BufReader::new(file);
                Ok(WeightEvaluator::read(buf)?)
            } else {
                eprintln!("Evaluator data not found: {}", data_path.display());
                Ok(WeightEvaluator::new())
            }
        }()
        .unwrap();

        Some(Computer::Ai(Arc::new(com), Arc::new(evaluator)))
    }
}

impl PlayState {
    pub(super) fn new(config: ConfigState) -> Self {
        let computer1 = Computer::from_config(config.player1());
        let computer2 = Computer::from_config(config.player2());
        Self {
            config,
            computer1,
            computer2,
            game: Game::new(),
            last_put: None,
            messages: vec![],
            state: GameState::Init,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) -> Option<super::GameState> {
        if matches!(self.state, GameState::Init) {
            self.update_state(ui);
        }
        self.check_status_updated(ui);

        let is_human_turn = matches!(self.state, GameState::WaitHuman);
        if let Some(pos) = board::show(ui, &self.game, is_human_turn, self.last_put) {
            self.put(ui, pos);
        }

        None
    }

    fn check_status_updated(&mut self, ui: &mut egui::Ui) {
        match &mut self.state {
            GameState::Init => {}
            GameState::WaitHuman => {}
            GameState::WaitComputer(rx) => match rx.try_recv() {
                Ok(next_move) => self.put(ui, next_move.chosen.unwrap().0),
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => panic!(),
            },
            GameState::GameOver => {}
        }
    }

    fn put(&mut self, ui: &mut egui::Ui, pos: Pos) {
        match self.game.put_disk(pos) {
            Ok(()) => {
                self.last_put = Some(pos);
                self.update_state(ui);
            }
            Err(e) => {
                self.messages.push(e.to_string());
                ui.ctx().request_repaint();
            }
        }
    }

    fn update_state(&mut self, ui: &mut egui::Ui) {
        let color = match self.game.turn_color() {
            Some(color) => color,
            None => {
                self.state = GameState::GameOver;
                return;
            }
        };

        let com = match color {
            Color::Black => &self.computer1,
            Color::White => &self.computer2,
        };

        match com {
            Some(Computer::Ai(com, evaluator)) => {
                let com = com.clone();
                let evaluator = evaluator.clone();
                let board = *self.game.board();
                let ctx = ui.ctx().clone();
                let (tx, rx) = mpsc::channel();
                thread::spawn(move || {
                    tx.send(com.next_move(&*evaluator, &board)).unwrap();
                    ctx.request_repaint();
                });
                self.state = GameState::WaitComputer(rx);
            }
            Some(Computer::Random) => {
                let mut rng = rand::thread_rng();
                let pos = self
                    .game
                    .board()
                    .flip_candidates()
                    .into_iter()
                    .choose(&mut rng)
                    .unwrap();
                self.put(ui, pos);
            }
            None => self.state = GameState::WaitHuman,
        };
    }
}
