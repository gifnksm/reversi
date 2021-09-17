use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    iter,
    path::PathBuf,
};

const BOARD_SIZE: i8 = 8;

type Error = Box<dyn std::error::Error>;
type Pos = (i8, i8);

fn pos_to_str((x, y): Pos) -> String {
    assert!((0..BOARD_SIZE).contains(&x));
    assert!((0..BOARD_SIZE).contains(&y));

    let alpha = (x as u8 + b'A') as char;
    let num = y + 1;
    format!("{}{}", alpha, num)
}

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    flip_lines(&mut File::create(&out_dir.join("flip_lines.rs"))?)?;

    Ok(())
}

fn flip_lines(file: &mut File) -> Result<(), Error> {
    let mut writer = BufWriter::new(file);

    writeln!(writer, "mod flip_lines {{")?;
    writeln!(writer, "    use super::{{FlipLines, Pos, PosSet}};")?;

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let pos = (x, y);
            let mut lines = vec![];
            for dy in [-1, 0, 1] {
                for dx in [-1, 0, 1] {
                    if (dx, dy) == (0, 0) {
                        continue;
                    }
                    let line = iter::successors(Some((x, y)), move |(x, y)| Some((x + dx, y + dy)))
                        .skip(1)
                        .take_while(|(x, y)| {
                            (0..BOARD_SIZE).contains(x) && (0..BOARD_SIZE).contains(y)
                        })
                        .map(|p| format!("Pos::{}", pos_to_str(p)))
                        .collect::<Vec<_>>();
                    if line.len() < 2 {
                        continue;
                    }
                    lines.push(line);
                }
            }

            writeln!(
                writer,
                "    const FLIP_LINE_{}: FlipLines = FlipLines {{",
                pos_to_str(pos)
            )?;
            writeln!(writer, "        pos: Pos::{},", pos_to_str(pos))?;
            writeln!(writer, "        lines: &[")?;
            for line in &lines {
                writeln!(writer, "            &[{}],", line.join(", "))?;
            }
            writeln!(writer, "        ],")?;
            writeln!(
                writer,
                "        self_mask: PosSet::from_slice(&[{}]),",
                lines
                    .iter()
                    .map(|line| line[1..].join(", "))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            writeln!(
                writer,
                "        other_mask: PosSet::from_slice(&[{}]),",
                lines
                    .iter()
                    .map(|line| line[..line.len() - 1].join(", "))
                    .collect::<Vec<String>>()
                    .join(", ")
            )?;
            writeln!(writer, "    }};")?;
        }
    }

    writeln!(
        writer,
        "    pub(super) fn flip_lines(p: Pos) -> &'static FlipLines {{"
    )?;
    writeln!(writer, "        match p {{")?;
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let pos = pos_to_str((x, y));
            writeln!(writer, "            Pos::{} => &FLIP_LINE_{},", pos, pos)?;
        }
    }
    writeln!(writer, "            _ => unreachable!(),")?;
    writeln!(writer, "        }}")?;
    writeln!(writer, "    }}")?;
    writeln!(writer, "}}")?;

    Ok(())
}
