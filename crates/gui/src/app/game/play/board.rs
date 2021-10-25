use eframe::egui::{self, Align2, Color32, Stroke, TextStyle, Ui, Vec2};
use reversi_core::{Board, Color, Game, Pos};

pub(super) fn show(
    ui: &mut Ui,
    game: &Game,
    is_human_turn: bool,
    last_put: Option<Pos>,
) -> Option<Pos> {
    let ctx = ui.ctx();
    let fonts = ctx.fonts();
    let text_style = TextStyle::Heading;
    let text_color = ui.visuals().text_color();

    let margin = ('1'..)
        .take(Board::SIZE as usize)
        .map(|ch| fonts.glyph_width(text_style, ch))
        .chain(
            ('A'..)
                .take(Board::SIZE as usize)
                .map(|ch| fonts.glyph_width(text_style, ch)),
        )
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let margin = Vec2::new(margin, margin);

    let sense = if is_human_turn {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };

    let (resp, painter) = ui.allocate_painter(2. * margin + BOARD_SIZE, sense);
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
        painter.text(
            origin
                + margin
                + Vec2::new(
                    CELL_SIZE.x * (x as f32 + 0.5),
                    BOARD_SIZE.y + margin.y / 2.0,
                ),
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
        painter.text(
            origin
                + margin
                + Vec2::new(
                    BOARD_SIZE.x + margin.x / 2.0,
                    CELL_SIZE.y * (y as f32 + 0.5),
                ),
            Align2::CENTER_CENTER,
            ch,
            text_style,
            text_color,
        );
    }

    let hover_disk_pos = resp
        .hover_pos()
        .and_then(|pos| to_disk_pos(pos - origin - margin));
    let hover_flipped_set = hover_disk_pos.and_then(|pos| game.board().flipped_set(pos));
    let clicked_disk_pos = resp
        .interact_pointer_pos()
        .and_then(|pos| to_disk_pos(pos - origin - margin));

    // disk
    let turn_color = game.turn_color();
    for (pos, disk) in game.pos_disks() {
        let mut circle = None;

        if let Some(color) = disk {
            circle = match color {
                Color::Black => Some(DISK_BLACK),
                Color::White => Some(DISK_WHITE),
            };
        }

        if is_human_turn {
            if let Some(turn_color) = turn_color {
                if game.board().can_flip(pos) {
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

        let center = origin
            + margin
            + Vec2::new(
                CELL_SIZE.x * (pos.x() as f32 + 0.5),
                CELL_SIZE.y * (pos.y() as f32 + 0.5),
            );

        if let Some((fill, stroke)) = circle {
            painter.circle(center, DISK_RADIUS, fill, stroke);
        }

        if hover_flipped_set
            .map(|flipped| flipped.contains(&pos))
            .unwrap_or_default()
        {
            painter.circle_stroke(center, FLIP_CANDIDATE_RADIUS, FLIP_CANDIDATE_STROKE);
        }

        if Some(pos) == last_put {
            painter.circle_filled(center, PUT_MARKER_RADIUS, PUT_MARKER_FILL);
        }
    }

    clicked_disk_pos
}

pub(super) const CELL_SIZE: Vec2 = Vec2::splat(32.0);
const BOARD_BG_COLOR: Color32 = Color32::from_rgb(0x00, 0x80, 0x00);
const BOARD_STROKE_COLOR: Color32 = Color32::BLACK;
const BOARD_STROKE: Stroke = Stroke {
    width: 1.,
    color: BOARD_STROKE_COLOR,
};
const DOT_RADIUS: f32 = 3.0;
pub(super) const DISK_RADIUS: f32 = 14.0;
pub(super) const DISK_BLACK: (Color32, Stroke) = (
    Color32::BLACK,
    Stroke {
        width: 0.1,
        color: Color32::BLACK,
    },
);
pub(super) const DISK_WHITE: (Color32, Stroke) = (
    Color32::WHITE,
    Stroke {
        width: 0.1,
        color: Color32::BLACK,
    },
);
pub(super) const BOARD_SIZE: Vec2 = Vec2::new(
    (CELL_SIZE.x) * Board::SIZE as f32,
    (CELL_SIZE.y) * Board::SIZE as f32,
);
const PUT_MARKER_RADIUS: f32 = 3.0;
const PUT_MARKER_FILL: Color32 = Color32::RED;
const FLIP_CANDIDATE_STROKE: Stroke = Stroke {
    width: 3.0,
    color: Color32::RED,
};
const FLIP_CANDIDATE_RADIUS: f32 = DISK_RADIUS;

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
