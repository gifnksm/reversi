use super::{CountEvaluator, Evaluate, DISK_VALUE};
use reversi_core::{Board, Pos};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::io::{Read, Write};

const UPDATE_RATIO: f64 = 0.005;
const MAX_PATTERN_VALUE: i16 = DISK_VALUE * 20;
const FREQ_THRESHOLD: u8 = 10;

fn update_value(value: &mut i16, count: u8, diff_sum: i32) {
    let updated =
        i32::from(*value) + (((diff_sum / i32::from(count)) as f64) * UPDATE_RATIO) as i32;
    *value = i32::clamp(
        updated,
        i32::from(-MAX_PATTERN_VALUE),
        i32::from(MAX_PATTERN_VALUE),
    ) as i16;
}

trait Pattern<const N: usize, const M: usize> {
    const PATTERNS: &'static [[Pos; N]];
    const WEIGHT_INDEX_OFFSET: usize;
    const WEIGHT_COUNT: usize;
    const PATTERN_TO_WEIGHT_MAP: &'static [u16; M];

    fn patterns() -> Vec<Vec<Pos>> {
        Self::PATTERNS
            .iter()
            .map(|pattern| pattern.to_vec())
            .collect()
    }

    fn weight(weight: &Weight) -> &[i16] {
        &weight.pattern[Self::WEIGHT_INDEX_OFFSET..][..Self::WEIGHT_COUNT]
    }

    fn pattern_to_weight_map() -> &'static [u16] {
        Self::PATTERN_TO_WEIGHT_MAP
    }

    fn evaluate(board: &Board, weight: &Weight) -> i32 {
        let weight = &weight.pattern[Self::WEIGHT_INDEX_OFFSET..][..Self::WEIGHT_COUNT];

        let mut value = 0;
        for pattern in Self::PATTERNS {
            let pattern_index = board.pattern_index(pattern);
            let weight_index = usize::from(Self::PATTERN_TO_WEIGHT_MAP[usize::from(pattern_index)]);
            value += i32::from(weight[weight_index]);
        }
        value
    }

    fn update(board: &Board, updater: &mut WeightUpdater, diff: i32) {
        let count = &mut updater.pattern_count[Self::WEIGHT_INDEX_OFFSET..][..Self::WEIGHT_COUNT];
        let sum = &mut updater.pattern_sum[Self::WEIGHT_INDEX_OFFSET..][..Self::WEIGHT_COUNT];

        for pattern in Self::PATTERNS {
            let pattern_index = board.pattern_index(pattern);
            let weight_index = usize::from(Self::PATTERN_TO_WEIGHT_MAP[usize::from(pattern_index)]);
            count[weight_index] += 1;
            sum[weight_index] += diff;
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/pattern.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weight {
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

impl Weight {
    pub fn patterns<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'static str, Vec<Vec<Pos>>, &'a [i16], &'static [u16])> + 'a {
        pattern::NAMES
            .iter()
            .copied()
            .zip(pattern::PATTERNS_FNS)
            .zip(pattern::WEIGHT_FNS)
            .zip(pattern::PATTERN_TO_WEIGHT_MAP_FNS)
            .map(move |(((name, pattern), weight), pattern_to_weight)| {
                (name, pattern(), weight(self), pattern_to_weight())
            })
    }

    pub fn parity(&self) -> &[i16; 2] {
        &self.parity
    }
}

fn board_parity_index(board: &Board) -> usize {
    (board.count_disk(None) % 2) as usize
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

    pub fn weight(&self) -> &Weight {
        &self.weight
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
            res += evaluate(board, &self.weight);
        }
        res += i32::from(self.weight.parity[board_parity_index(board)]);

        res
    }
}

impl Evaluate for WeightEvaluator {
    fn evaluate(&self, board: &Board, game_over: bool) -> i32 {
        if game_over {
            self.count_evaluator.evaluate(board, game_over)
        } else {
            self.compute_value(board)
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeightUpdater {
    evaluator: WeightEvaluator,
    pattern_count: [u8; pattern::WEIGHT_COUNT],
    pattern_sum: [i32; pattern::WEIGHT_COUNT],
    parity_count: [u8; 2],
    parity_sum: [i32; 2],
}

impl WeightUpdater {
    pub fn new(evaluator: WeightEvaluator) -> Self {
        Self {
            evaluator,
            pattern_count: [0; pattern::WEIGHT_COUNT],
            pattern_sum: [0; pattern::WEIGHT_COUNT],
            parity_count: [0; 2],
            parity_sum: [0; 2],
        }
    }

    pub fn evaluator(&self) -> &WeightEvaluator {
        &self.evaluator
    }

    pub fn update(&mut self, board: &Board, value: i32) -> i32 {
        let diff = value - self.evaluator.compute_value(board);
        for update in pattern::UPDATE_FNS {
            update(board, self, diff);
        }

        let parity_index = board_parity_index(board);
        self.parity_count[parity_index] += 1;
        self.parity_sum[parity_index] += diff;

        diff
    }

    pub fn flush(&mut self) {
        fn inner<const N: usize>(count: &mut [u8; N], sum: &mut [i32; N], weight: &mut [i16; N]) {
            count
                .iter_mut()
                .zip(sum)
                .zip(weight)
                .filter(|((count, _), _)| **count > FREQ_THRESHOLD)
                .for_each(|((count, sum), weight)| {
                    update_value(weight, *count, *sum);
                    *count = 0;
                    *sum = 0;
                });
        }

        inner(
            &mut self.pattern_count,
            &mut self.pattern_sum,
            &mut self.evaluator.weight.pattern,
        );
        inner(
            &mut self.parity_count,
            &mut self.parity_sum,
            &mut self.evaluator.weight.parity,
        );
    }
}
