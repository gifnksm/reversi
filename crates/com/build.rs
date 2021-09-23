use reversi_core::{Board, Pos, PosSet};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{prelude::*, BufWriter},
    path::PathBuf,
};
use Pos as P;

type Error = Box<dyn std::error::Error>;

const PATTERNS: &[(&str, &[Pos])] = &[
    ("Diag4", &[P::D1, P::C2, P::B3, P::A4]),
    ("Diag5", &[P::E1, P::D2, P::C3, P::B4, P::A5]),
    ("Diag6", &[P::F1, P::E2, P::D3, P::C4, P::B5, P::A6]),
    ("Diag7", &[P::G1, P::F2, P::E3, P::D4, P::C5, P::B6, P::A7]),
    (
        "Diag8",
        &[P::H1, P::G2, P::F3, P::E4, P::D5, P::C6, P::B7, P::A8],
    ),
    (
        "Line2",
        &[P::A2, P::B2, P::C2, P::D2, P::E2, P::F2, P::G2, P::H2],
    ),
    (
        "Line3",
        &[P::A3, P::B3, P::C3, P::D3, P::E3, P::F3, P::G3, P::H3],
    ),
    (
        "Line4",
        &[P::A4, P::B4, P::C4, P::D4, P::E4, P::F4, P::G4, P::H4],
    ),
    (
        "Edge",
        &[
            P::A1,
            P::B1,
            P::C1,
            P::D1,
            P::E1,
            P::F1,
            P::G1,
            P::H1,
            P::B2,
            P::G2,
        ],
    ),
    (
        "Corner3x3",
        &[
            P::A1,
            P::B1,
            P::C1,
            P::A2,
            P::B2,
            P::C2,
            P::A3,
            P::B3,
            P::C3,
        ],
    ),
    (
        "Corner5x2",
        &[
            P::A1,
            P::B1,
            P::C1,
            P::D1,
            P::E1,
            P::A2,
            P::B2,
            P::C2,
            P::D2,
            P::E2,
        ],
    ),
];

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(&std::env::var_os("OUT_DIR").unwrap());
    let file = File::create(out_dir.join("pattern.rs"))?;

    let mut writer = BufWriter::new(file);
    writeln!(&mut writer, "mod pattern {{")?;
    writeln!(&mut writer, "    use reversi_core::{{Board, Pos}};")?;
    writeln!(
        &mut writer,
        "    use super::{{Pattern, Weight, WeightUpdater}};"
    )?;

    let mut weight_index = 0;
    let mut pattern_to_weight_map_list = vec![];
    for (name, pattern) in PATTERNS {
        emit_pattern(
            &mut writer,
            &mut weight_index,
            &mut pattern_to_weight_map_list,
            name,
            pattern,
        )?;
    }

    writeln!(&mut writer, "pub(super) const NAMES: &[&'static str] = &[")?;
    for (name, _) in PATTERNS {
        writeln!(&mut writer, "{:?},", name)?;
    }
    writeln!(&mut writer, "];")?;

    writeln!(
        &mut writer,
        "pub(super) const PATTERN_FNS: &[fn() -> Vec<Vec<Pos>>] = &["
    )?;
    for (name, _) in PATTERNS {
        writeln!(&mut writer, "{}::patterns,", name)?;
    }
    writeln!(&mut writer, "];")?;

    writeln!(
        &mut writer,
        "pub(super) const WEIGHT_FNS: &[fn(weight: &Weight) -> &[i16]] = &["
    )?;
    for (name, _) in PATTERNS {
        writeln!(&mut writer, "{}::weight,", name)?;
    }
    writeln!(&mut writer, "];")?;

    writeln!(
        &mut writer,
        "pub(super) const EVALUATE_FNS: &[fn (board: &Board, weight: &Weight) -> i32] = &["
    )?;
    for (name, _) in PATTERNS {
        writeln!(&mut writer, "{}::evaluate,", name)?;
    }
    writeln!(&mut writer, "];")?;

    writeln!(
        &mut writer,
        "pub(super) const UPDATE_FNS: &[fn (board: &Board, updater: &mut WeightUpdater, diff: i32)] = &["
    )?;
    for (name, _) in PATTERNS {
        writeln!(&mut writer, "{}::update,", name)?;
    }
    writeln!(&mut writer, "];")?;

    writeln!(
        &mut writer,
        "pub(super) const WEIGHT_COUNT: usize = {};",
        weight_index
    )?;
    for (i, map) in pattern_to_weight_map_list.iter().enumerate() {
        writeln!(
            &mut writer,
            "pub(super) const PATTERN_TO_WEIGHT_MAP_{}: [u16; {}] = {:?};",
            i,
            map.len(),
            map,
        )?;
    }
    writeln!(&mut writer, "}}")?;

    Ok(())
}

fn emit_pattern(
    mut writer: impl Write,
    weight_index: &mut u32,
    pattern_to_weight_map_list: &mut Vec<Vec<u16>>,
    name: &str,
    pattern: &[Pos],
) -> Result<(), Error> {
    let pattern_index_count = 3u16.pow(pattern.len() as u32);
    let pattern_map = PatternMap::from_pattern(pattern.into());

    let weight_index_offset = *weight_index;

    let (weight_count, pattern_to_weight_map_index) = create_pattern_to_weight_map(
        pattern_index_count,
        &pattern_map,
        pattern_to_weight_map_list,
    );
    *weight_index += u32::from(weight_count);

    writeln!(&mut writer, "    pub(super) struct {};", name)?;
    writeln!(
        &mut writer,
        "    impl Pattern<{}, {}> for {} {{",
        pattern.len(),
        pattern_index_count,
        name
    )?;
    writeln!(
        &mut writer,
        "        const PATTERNS: &'static [[Pos; {}]] = &[",
        pattern.len()
    )?;
    for set in pattern_map.0.values() {
        let pattern = set.iter().next().unwrap();
        write!(&mut writer, "            [")?;
        for pos in pattern {
            write!(&mut writer, "Pos::{}, ", pos)?;
        }
        writeln!(&mut writer, "],")?;
    }
    writeln!(&mut writer, "        ];")?;
    writeln!(
        &mut writer,
        "        const WEIGHT_INDEX_OFFSET: usize = {};",
        weight_index_offset
    )?;
    writeln!(
        &mut writer,
        "        const WEIGHT_COUNT: usize = {};",
        weight_count
    )?;
    writeln!(
        &mut writer,
        "        const PATTERN_TO_WEIGHT_MAP: &'static [u16; {}] = &PATTERN_TO_WEIGHT_MAP_{};",
        pattern_index_count, pattern_to_weight_map_index,
    )?;

    writeln!(&mut writer, "    }}")?;

    Ok(())
}

#[derive(Debug, Clone, Default)]
struct PatternMap(BTreeMap<PosSet, BTreeSet<Vec<Pos>>>);

impl PatternMap {
    fn from_pattern(pattern: Vec<Pos>) -> Self {
        let mut pattern_map = Self::default();
        pattern_map.insert(pattern);

        const MAX: i8 = Board::SIZE - 1;

        fn insert_mapped(map: &mut PatternMap, f: impl Fn(Pos) -> Pos) {
            for pattern in map.all_patterns() {
                let flipped = pattern.into_iter().map(&f).collect();
                map.insert(flipped);
            }
        }

        // horizontal flipped
        insert_mapped(&mut pattern_map, |p| {
            Pos::from_xy(MAX - p.x(), p.y()).unwrap()
        });

        // vertical flipped
        insert_mapped(&mut pattern_map, |p| {
            Pos::from_xy(p.x(), MAX - p.y()).unwrap()
        });

        // rotate 90 deg 3times
        for _ in 0..3 {
            insert_mapped(&mut pattern_map, |p| {
                Pos::from_xy(MAX - p.y(), p.x()).unwrap()
            });
        }

        pattern_map
    }

    fn insert(&mut self, pattern: Vec<Pos>) {
        let set = pattern.iter().copied().collect::<PosSet>();
        self.0.entry(set).or_default().insert(pattern);
    }

    fn all_patterns(&self) -> Vec<Vec<Pos>> {
        self.0
            .iter()
            .flat_map(|(_, set)| set.iter().cloned())
            .collect()
    }
}

fn create_pattern_to_weight_map(
    pattern_index_count: u16,
    pattern_map: &PatternMap,
    pattern_to_weight_map_list: &mut Vec<Vec<u16>>,
) -> (u16, usize) {
    let mut weight_count = 0;
    let (_, first_patterns) = pattern_map.0.iter().next().unwrap();
    let mut pattern_to_weight_map = vec![u16::MAX; usize::from(pattern_index_count)]; // pattern index -> weight index
    for i in 0..pattern_index_count {
        let mut first_patterns = first_patterns.iter();
        let first_pattern = first_patterns.next().unwrap();
        if pattern_to_weight_map[usize::from(i)] != u16::MAX {
            continue;
        }
        let store_index = weight_count;
        weight_count += 1;
        pattern_to_weight_map[usize::from(i)] = store_index;
        for pattern in first_patterns {
            let board = Board::from_pattern_index(pattern, i as u16);
            pattern_to_weight_map[usize::from(board.pattern_index(first_pattern))] = store_index;
        }
    }

    assert_eq!(
        pattern_to_weight_map.len(),
        usize::from(pattern_index_count)
    );

    let pattern_to_weight_map_index = match pattern_to_weight_map_list
        .iter()
        .position(|elem| elem == &pattern_to_weight_map)
    {
        Some(idx) => idx,
        None => {
            pattern_to_weight_map_list.push(pattern_to_weight_map);
            pattern_to_weight_map_list.len() - 1
        }
    };

    (weight_count, pattern_to_weight_map_index)
}
