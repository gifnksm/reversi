use super::config::ConfigState;
use crate::player::{AiLevel, ComputerKind, PlayerConf, PlayerKind};
use eframe::egui::{self, Align2, Color32, Stroke, TextStyle, Vec2};
use rand::prelude::*;
use reversi_com::{Com, NextMove, WeightEvaluator};
use reversi_core::{Board, Color, Game, Pos};
use std::{
    fs::File,
    io::BufReader,
    path::Path,
    sync::{mpsc, Arc},
    thread,
};

#[derive(Debug)]
pub(super) struct PlayState {
    config: ConfigState,
    computer1: Option<Computer>,
    computer2: Option<Computer>,
    game: Game,
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
            messages: vec![],
            state: GameState::Init,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut egui::Ui) -> Option<super::GameState> {
        if matches!(self.state, GameState::Init) {
            self.update_state(ui);
        }
        self.check_status_updated(ui);
        self.ui_board(ui);
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
            Ok(()) => self.update_state(ui),
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

    fn ui_board(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx();
        let fonts = ctx.fonts();
        let text_style = TextStyle::Heading;
        let text_color = ui.visuals().text_color();

        let is_human_turn = matches!(self.state, GameState::WaitHuman);

        let margin = ('1'..)
            .take(Board::SIZE as usize)
            .map(|ch| fonts.layout_no_wrap(text_style, ch.into()).size.x)
            .chain(
                ('A'..)
                    .take(Board::SIZE as usize)
                    .map(|ch| fonts.layout_no_wrap(text_style, ch.into()).size.y),
            )
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let margin = Vec2::new(margin, margin);

        let sense = if is_human_turn {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        };

        let (resp, painter) = ui.allocate_painter(margin + BOARD_SIZE, sense);
        let origin = painter.clip_rect().min;

        // background
        painter.rect_filled(
            egui::Rect::from_min_size(origin + margin, BOARD_SIZE),
            0.0,
            BOARD_BG_COLOR,
        );

        // stroke
        for idx in 0..=(Board::SIZE) {
            let idx = idx as f32;
            painter.line_segment(
                [
                    origin + margin + Vec2::new(CELL_SIZE.x * idx, 0.),
                    origin + margin + Vec2::new(CELL_SIZE.x * idx, BOARD_SIZE.y),
                ],
                BOARD_STROKE,
            );
            painter.line_segment(
                [
                    origin + margin + Vec2::new(0., CELL_SIZE.y * idx),
                    origin + margin + Vec2::new(BOARD_SIZE.x, CELL_SIZE.y * idx),
                ],
                BOARD_STROKE,
            );
        }

        // dots
        for x in [2, Board::SIZE - 2] {
            for y in [2, Board::SIZE - 2] {
                painter.circle_filled(
                    origin + margin + Vec2::new(CELL_SIZE.x * x as f32, CELL_SIZE.y * y as f32),
                    DOT_RADIUS,
                    BOARD_STROKE_COLOR,
                );
            }
        }

        // coordination label
        for (ch, x) in ('A'..).take(Board::SIZE as usize).zip(0..) {
            painter.text(
                origin + margin + Vec2::new(CELL_SIZE.x * (x as f32 + 0.5), -margin.y / 2.0),
                Align2::CENTER_CENTER,
                ch,
                text_style,
                text_color,
            );
        }
        for (ch, y) in ('1'..).take(Board::SIZE as usize).zip(0..) {
            painter.text(
                origin + margin + Vec2::new(-margin.x / 2.0, CELL_SIZE.y * (y as f32 + 0.5)),
                Align2::CENTER_CENTER,
                ch,
                text_style,
                text_color,
            );
        }

        let hover_disk_pos = resp
            .hover_pos()
            .and_then(|pos| to_disk_pos(pos - origin - margin));
        let clicked_disk_pos = resp
            .interact_pointer_pos()
            .and_then(|pos| to_disk_pos(pos - origin - margin));

        // disk
        let turn_color = self.game.turn_color();
        for y in 0..Board::SIZE {
            for x in 0..Board::SIZE {
                let pos = Pos::from_xy(x, y).unwrap();
                let mut circle = None;

                if let Some(color) = self.game.get_disk(pos) {
                    circle = match color {
                        Color::Black => Some(DISK_BLACK),
                        Color::White => Some(DISK_WHITE),
                    };
                }

                if is_human_turn {
                    if let Some(turn_color) = turn_color {
                        if self.game.board().can_flip(pos) {
                            let alpha = if hover_disk_pos == Some(pos) {
                                0.8
                            } else {
                                0.2
                            };
                            let (mut fill, mut stroke) = match turn_color {
                                Color::Black => DISK_BLACK,
                                Color::White => DISK_WHITE,
                            };
                            fill = mix_color(fill, BOARD_BG_COLOR, alpha);
                            stroke.color = mix_color(stroke.color, BOARD_BG_COLOR, alpha);
                            circle = Some((fill, stroke));
                        }
                    }
                }

                if let Some((fill, stroke)) = circle {
                    let center = origin
                        + margin
                        + Vec2::new(
                            CELL_SIZE.x * (x as f32 + 0.5),
                            CELL_SIZE.y * (y as f32 + 0.5),
                        );
                    painter.circle(center, DISK_RADIUS, fill, stroke);
                }
            }
        }

        // flip disk
        if let Some(pos) = clicked_disk_pos {
            self.put(ui, pos);
        }
    }
}

const CELL_SIZE: Vec2 = Vec2::new(32.0, 32.0);
const BOARD_BG_COLOR: Color32 = Color32::from_rgb(0x00, 0x80, 0x00);
const BOARD_STROKE_COLOR: Color32 = Color32::BLACK;
const BOARD_STROKE: Stroke = Stroke {
    width: 1.,
    color: BOARD_STROKE_COLOR,
};
const DOT_RADIUS: f32 = 3.0;
const DISK_RADIUS: f32 = 14.0;
const DISK_BLACK: (Color32, Stroke) = (
    Color32::BLACK,
    Stroke {
        width: 0.1,
        color: Color32::BLACK,
    },
);
const DISK_WHITE: (Color32, Stroke) = (
    Color32::WHITE,
    Stroke {
        width: 0.1,
        color: Color32::BLACK,
    },
);
const BOARD_SIZE: Vec2 = Vec2::new(
    (CELL_SIZE.x) * Board::SIZE as f32,
    (CELL_SIZE.y) * Board::SIZE as f32,
);

fn to_disk_pos(pos: Vec2) -> Option<Pos> {
    if pos.clamp(Vec2::ZERO, BOARD_SIZE) != pos {
        return None;
    }

    let cell_pos = (pos / CELL_SIZE).floor();
    let cell_pos = Pos::from_xy(cell_pos.x as i8, cell_pos.y as i8)?;
    let pos = pos
        - Vec2::new(
            CELL_SIZE.x * cell_pos.x() as f32,
            CELL_SIZE.y * cell_pos.y() as f32,
        );
    if (pos - CELL_SIZE / 2.0).length() > DISK_RADIUS {
        return None;
    }
    Some(cell_pos)
}

fn mix_color(a: Color32, b: Color32, alpha: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 * alpha + b.r() as f32 * (1.0 - alpha)) as u8,
        (a.g() as f32 * alpha + b.g() as f32 * (1.0 - alpha)) as u8,
        (a.b() as f32 * alpha + b.b() as f32 * (1.0 - alpha)) as u8,
    )
}
