use std::io::{Read, Write};

use reversi_core::{Board, Color, Pos};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use Pos as P;

const DISK_VALUE: i16 = 1000;

pub trait Evaluate {
    fn evaluate(&self, board: &Board, color: Color, game_over: bool) -> i32;
}

#[derive(Debug, Default, Clone)]
pub struct CountEvaluator {}

impl CountEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluate for CountEvaluator {
    fn evaluate(&self, board: &Board, color: Color, _game_over: bool) -> i32 {
        i32::from(DISK_VALUE)
            * ((board.count(Some(color)) as i32) - (board.count(Some(color.reverse())) as i32))
    }
}

const UPDATE_RATIO: f64 = 0.003;
const MAX_PATTERN_VALUE: i16 = DISK_VALUE * 20;

const POW_3_0: usize = 3usize.pow(0);
const POW_3_1: usize = 3usize.pow(1);
const POW_3_2: usize = 3usize.pow(2);
const POW_3_3: usize = 3usize.pow(3);
const POW_3_4: usize = 3usize.pow(4);
const POW_3_5: usize = 3usize.pow(5);
const POW_3_6: usize = 3usize.pow(6);
const POW_3_7: usize = 3usize.pow(7);
const POW_3_8: usize = 3usize.pow(8);

#[derive(Debug, Clone, Copy)]
enum Pattern {
    Line4,
    Line3,
    Line2,
    Diag8,
    Diag7,
    Diag6,
    Diag5,
    Diag4,
    Edge,
    Corner,
    Parity,
}

impl Pattern {
    const fn size(&self) -> usize {
        use Pattern::*;
        match self {
            Line4 | Line3 | Line2 | Diag8 => POW_3_8,
            Diag7 => POW_3_7,
            Diag6 => POW_3_6,
            Diag5 => POW_3_5,
            Diag4 => POW_3_4,
            Edge => POW_3_8,
            Corner => POW_3_8,
            Parity => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Weight {
    #[serde(with = "BigArray")]
    line4: [i16; Pattern::Line4.size()],
    #[serde(with = "BigArray")]
    line3: [i16; Pattern::Line3.size()],
    #[serde(with = "BigArray")]
    line2: [i16; Pattern::Line2.size()],
    #[serde(with = "BigArray")]
    diag8: [i16; Pattern::Diag8.size()],
    #[serde(with = "BigArray")]
    diag7: [i16; Pattern::Diag7.size()],
    #[serde(with = "BigArray")]
    diag6: [i16; Pattern::Diag6.size()],
    #[serde(with = "BigArray")]
    diag5: [i16; Pattern::Diag5.size()],
    #[serde(with = "BigArray")]
    diag4: [i16; Pattern::Diag4.size()],
    #[serde(with = "BigArray")]
    edge: [i16; Pattern::Edge.size()],
    #[serde(with = "BigArray")]
    corner: [i16; Pattern::Corner.size()],
    parity: [i16; Pattern::Parity.size()],
}

impl Default for Weight {
    fn default() -> Self {
        Self {
            line4: [0; Pattern::Line4.size()],
            line3: [0; Pattern::Line3.size()],
            line2: [0; Pattern::Line2.size()],
            diag8: [0; Pattern::Diag8.size()],
            diag7: [0; Pattern::Diag7.size()],
            diag6: [0; Pattern::Diag6.size()],
            diag5: [0; Pattern::Diag5.size()],
            diag4: [0; Pattern::Diag4.size()],
            edge: [0; Pattern::Edge.size()],
            corner: [0; Pattern::Corner.size()],
            parity: [0; Pattern::Parity.size()],
        }
    }
}

fn cell_index(cell: Option<Color>) -> usize {
    match cell {
        Some(Color::Black) => 1,
        Some(Color::White) => 2,
        None => 0,
    }
}

fn board_index(board: &Board, pos: &[Pos]) -> usize {
    pos.iter()
        .copied()
        .fold(0, |acc, x| acc + cell_index(board.get(x)) * 3)
}

fn board_parity_index(board: &Board) -> usize {
    (board.count(None) % 2) as usize
}

const LINE4_POS: &[&[Pos]] = &[
    &[P::A4, P::B4, P::C4, P::D4, P::E4, P::F4, P::G4, P::H4],
    &[P::A5, P::B5, P::C5, P::D5, P::E5, P::F5, P::G5, P::H5],
    &[P::D1, P::D2, P::D3, P::D4, P::D5, P::D6, P::D7, P::D8],
    &[P::E1, P::E2, P::E3, P::E4, P::E5, P::E6, P::E7, P::E8],
];
const LINE3_POS: &[&[Pos]] = &[
    &[P::A3, P::B3, P::C3, P::D3, P::E3, P::F3, P::G3, P::H3],
    &[P::A6, P::B6, P::C6, P::D6, P::E6, P::F6, P::G6, P::H6],
    &[P::C1, P::C2, P::C3, P::C4, P::C5, P::C6, P::C7, P::C8],
    &[P::F1, P::F2, P::F3, P::F4, P::F5, P::F6, P::F7, P::F8],
];
const LINE2_POS: &[&[Pos]] = &[
    &[P::A2, P::B2, P::C2, P::D2, P::E2, P::F2, P::G2, P::H2],
    &[P::A7, P::B7, P::C7, P::D7, P::E7, P::F7, P::G7, P::H7],
    &[P::B1, P::B2, P::B3, P::B4, P::B5, P::B6, P::B7, P::B8],
    &[P::G1, P::G2, P::G3, P::G4, P::G5, P::G6, P::G7, P::G8],
];
const DIAG8_POS: &[&[Pos]] = &[
    &[P::A1, P::B2, P::C3, P::D4, P::E5, P::F6, P::G7, P::H8],
    &[P::A8, P::B7, P::C6, P::D5, P::E4, P::F3, P::G2, P::H1],
];
const DIAG7_POS: &[&[Pos]] = &[
    &[P::A2, P::B3, P::C4, P::D5, P::E6, P::F7, P::G8],
    &[P::B1, P::C2, P::D3, P::E4, P::F5, P::G6, P::H7],
    &[P::A7, P::B6, P::C5, P::D4, P::E3, P::F2, P::G1],
    &[P::B8, P::C7, P::D6, P::E5, P::F4, P::G3, P::H2],
];
const DIAG6_POS: &[&[Pos]] = &[
    &[P::A3, P::B4, P::C5, P::D6, P::E7, P::F8],
    &[P::C1, P::D2, P::E3, P::F4, P::G5, P::H6],
    &[P::A6, P::B5, P::C4, P::D3, P::E2, P::F1],
    &[P::C8, P::D7, P::E6, P::F5, P::G4, P::H3],
];
const DIAG5_POS: &[&[Pos]] = &[
    &[P::A4, P::B5, P::C6, P::D7, P::E8],
    &[P::D1, P::E2, P::F3, P::G4, P::H5],
    &[P::A5, P::B4, P::C3, P::D2, P::E1],
    &[P::D8, P::E7, P::F6, P::G5, P::H4],
];
const DIAG4_POS: &[&[Pos]] = &[
    &[P::A5, P::B6, P::C7, P::D8],
    &[P::E1, P::F2, P::G3, P::H4],
    &[P::A4, P::B3, P::C2, P::D1],
    &[P::E8, P::F7, P::G6, P::H5],
];
const EDGE_POS: &[&[Pos]] = &[
    &[P::A1, P::B1, P::C1, P::D1, P::E1, P::F1, P::G1, P::B2],
    &[P::H1, P::G1, P::F1, P::E1, P::D1, P::C1, P::B1, P::G2],
    &[P::A8, P::B8, P::C8, P::D8, P::E8, P::F8, P::G8, P::B7],
    &[P::H8, P::G8, P::F8, P::E8, P::D8, P::C8, P::B8, P::G7],
    &[P::A1, P::A2, P::A3, P::A4, P::A5, P::A6, P::A7, P::B2],
    &[P::A8, P::A7, P::A6, P::A5, P::A4, P::A3, P::A2, P::B7],
    &[P::H1, P::H2, P::H3, P::H4, P::H5, P::H6, P::H7, P::G2],
    &[P::H8, P::H7, P::H6, P::H5, P::H4, P::H3, P::H2, P::G7],
];
const CORNER_POS: &[&[Pos]] = &[
    &[P::A1, P::B1, P::C1, P::A2, P::B2, P::C2, P::A3, P::B3],
    &[P::H1, P::G1, P::F1, P::H2, P::G2, P::F2, P::H3, P::G3],
    &[P::A8, P::B8, P::C8, P::A7, P::B7, P::C7, P::A6, P::B6],
    &[P::H8, P::G8, P::F8, P::H7, P::G7, P::F7, P::H6, P::G6],
];

#[derive(Debug, Default, Clone)]
pub struct WeightEvaluator {
    count_evaluator: CountEvaluator,
    weight: Box<Weight>,
}

impl WeightEvaluator {
    pub fn new() -> Self {
        Self::default()
    }

    fn with_weight(weight: Box<Weight>) -> Self {
        Self {
            weight,
            ..Default::default()
        }
    }

    pub fn read(reader: impl Read) -> bincode::Result<Self> {
        Ok(Self::with_weight(bincode::deserialize_from(reader)?))
    }

    pub fn write(&self, writer: impl Write) -> bincode::Result<()> {
        bincode::serialize_into(writer, &self.weight)
    }

    fn compute_value(&self, board: &Board) -> i32 {
        let mut res = 0;

        fn value(board: &Board, weight: &[i16], pos: &[Pos]) -> i32 {
            assert_eq!(weight.len(), 3usize.pow(pos.len() as u32));
            i32::from(weight[board_index(board, pos)])
        }

        let list: &[(&[_], _)] = &[
            (&self.weight.line4, LINE4_POS),
            (&self.weight.line3, LINE3_POS),
            (&self.weight.line2, LINE2_POS),
            (&self.weight.diag8, DIAG8_POS),
            (&self.weight.diag7, DIAG7_POS),
            (&self.weight.diag6, DIAG6_POS),
            (&self.weight.diag5, DIAG5_POS),
            (&self.weight.diag4, DIAG4_POS),
            (&self.weight.edge, EDGE_POS),
            (&self.weight.corner, CORNER_POS),
        ];

        for &(weight, pos_list) in list {
            for pos in pos_list {
                res += value(board, weight, pos);
            }
        }

        res += i32::from(self.weight.parity[board_parity_index(board)]);

        res
    }
}

impl Evaluate for WeightEvaluator {
    fn evaluate(&self, board: &Board, color: Color, game_over: bool) -> i32 {
        if game_over {
            self.count_evaluator.evaluate(board, color, game_over)
        } else {
            self.compute_value(board)
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeightUpdater {
    evaluator: WeightEvaluator,
    mirror_line: Box<[usize; POW_3_8]>,
    mirror_corner: Box<[usize; POW_3_8]>,
}

impl WeightUpdater {
    pub fn new(evaluator: WeightEvaluator) -> Self {
        let mut res = Self {
            evaluator,
            mirror_line: Box::new([0; POW_3_8]),
            mirror_corner: Box::new([0; POW_3_8]),
        };

        let line_coeff = [
            POW_3_7, POW_3_6, POW_3_5, POW_3_4, POW_3_3, POW_3_2, POW_3_1, POW_3_0,
        ];
        for (mut i, value) in res.mirror_line.iter_mut().enumerate() {
            for &coeff in &line_coeff {
                *value += i % 3 * coeff;
                i /= 3;
            }
            *value = usize::min(i, *value);
        }

        let corner_coeff = [
            POW_3_2, POW_3_5, POW_3_0, POW_3_3, POW_3_6, POW_3_1, POW_3_4, POW_3_7,
        ];
        for (mut i, value) in res.mirror_corner.iter_mut().enumerate() {
            for &coeff in &corner_coeff {
                *value += i % 3 * coeff;
                i /= 3;
            }
            *value = usize::min(i, *value);
        }

        res
    }

    pub fn evaluator(&self) -> &WeightEvaluator {
        &self.evaluator
    }

    pub fn update(&mut self, board: &Board, value: i32) {
        fn update_pattern(weight: &mut [i16], idx: usize, mirror_idx: Option<usize>, diff: i32) {
            weight[idx] = i32::clamp(
                i32::from(weight[idx]) + diff,
                i32::from(-MAX_PATTERN_VALUE),
                i32::from(MAX_PATTERN_VALUE),
            ) as i16;
            if let Some(mirror_idx) = mirror_idx {
                weight[mirror_idx] = weight[idx];
            }
        }

        let diff =
            (value as f64 - (self.evaluator.compute_value(board) as f64) * UPDATE_RATIO) as i32;

        let w = &mut self.evaluator.weight;

        let line_patterns = &mut [
            (&mut w.line4, LINE4_POS),
            (&mut w.line3, LINE3_POS),
            (&mut w.line2, LINE2_POS),
        ];
        for (weight, pos_list) in line_patterns {
            for pos in *pos_list {
                let index = board_index(board, pos);
                update_pattern(*weight, self.mirror_line[index], Some(index), diff)
            }
        }

        let diag_patterns: &mut [(&mut [_], _, _)] = &mut [
            (&mut w.diag8, DIAG8_POS, POW_3_0),
            (&mut w.diag7, DIAG7_POS, POW_3_1),
            (&mut w.diag6, DIAG6_POS, POW_3_2),
            (&mut w.diag5, DIAG5_POS, POW_3_3),
            (&mut w.diag4, DIAG4_POS, POW_3_4),
        ];
        for (weight, pos_list, coeff) in diag_patterns {
            for pos in *pos_list {
                let index = board_index(board, pos);
                update_pattern(*weight, self.mirror_line[index * *coeff], Some(index), diff);
            }
        }

        for pos in EDGE_POS {
            let index = board_index(board, pos);
            update_pattern(&mut w.edge, index, None, diff);
        }

        for pos in CORNER_POS {
            let index = board_index(board, pos);
            update_pattern(&mut w.corner, self.mirror_corner[index], Some(index), diff);
        }

        let index = board_parity_index(board);
        update_pattern(&mut w.parity, index, None, diff);
    }
}
