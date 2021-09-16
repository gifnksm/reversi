use super::{CountEvaluator, Evaluate, DISK_VALUE};
use reversi_core::{Board, Color, Pos};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::io::{Read, Write};

const UPDATE_RATIO: f64 = 0.003;
const MAX_PATTERN_VALUE: i16 = DISK_VALUE * 20;

fn update_value(value: &mut i16, diff: i32) {
    *value = i32::clamp(
        i32::from(*value) + diff,
        i32::from(-MAX_PATTERN_VALUE),
        i32::from(MAX_PATTERN_VALUE),
    ) as i16;
}

trait Pattern<const N: usize, const M: usize> {
    const PATTERNS: &'static [[Pos; N]];
    const WEIGHT_INDEX_OFFSET: usize;
    const WEIGHT_COUNT: usize;
    const PATTERN_TO_WEIGHT_MAP: &'static [u16; M];

    fn evaluate(board: &Board, weight: &[i16]) -> i32 {
        let weight = &weight[Self::WEIGHT_INDEX_OFFSET..][..Self::WEIGHT_COUNT];

        let mut value = 0;
        for pattern in Self::PATTERNS {
            let pattern_index = board.pattern_index(pattern);
            let weight_index = usize::from(Self::PATTERN_TO_WEIGHT_MAP[usize::from(pattern_index)]);
            value += i32::from(weight[weight_index]);
        }
        value
    }

    fn update(board: &Board, weight: &mut [i16], diff: i32) {
        let weight = &mut weight[Self::WEIGHT_INDEX_OFFSET..][..Self::WEIGHT_COUNT];

        for pattern in Self::PATTERNS {
            let pattern_index = board.pattern_index(pattern);
            let weight_index = usize::from(Self::PATTERN_TO_WEIGHT_MAP[usize::from(pattern_index)]);
            update_value(&mut weight[weight_index], diff);
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/pattern.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Weight {
    #[serde(with = "BigArray")]
    pattern: [i16; pattern::WEIGHT_COUNT],
    parity: [i16; 2],
}

impl Default for Weight {
    fn default() -> Self {
        Self {
            pattern: [0; pattern::WEIGHT_COUNT],
            parity: [0; 2],
        }
    }
}

fn board_parity_index(board: &Board) -> usize {
    (board.count(None) % 2) as usize
}

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

        for evaluate in pattern::EVALUATE_FNS {
            res += evaluate(board, &self.weight.pattern);
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
}

impl WeightUpdater {
    pub fn new(evaluator: WeightEvaluator) -> Self {
        Self { evaluator }
    }

    pub fn evaluator(&self) -> &WeightEvaluator {
        &self.evaluator
    }

    pub fn update(&mut self, board: &Board, value: i32) {
        let diff =
            (value as f64 - (self.evaluator.compute_value(board) as f64) * UPDATE_RATIO) as i32;
        let w = &mut self.evaluator.weight;

        for update in pattern::UPDATE_FNS {
            update(board, &mut w.pattern, diff);
        }
        update_value(&mut w.parity[board_parity_index(board)], diff);
    }
}
