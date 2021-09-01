use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    iter,
    path::Path,
};

const BOARD_SIZE: i8 = 8;

type Pos = (i8, i8);

fn pos_to_str((x, y): Pos) -> String {
    assert!((0..BOARD_SIZE).contains(&x));
    assert!((0..BOARD_SIZE).contains(&y));

    let alpha = (x as u8 + b'A') as char;
    let num = y + 1;
    format!("{}{}", alpha, num)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("pos_lines.rs");
    let file = File::create(&dest_path)?;
    let mut writer = BufWriter::new(file);

    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let pos = (x, y);

            writeln!(
                writer,
                "const FLIP_LINE_{}: &[&[Pos]] = &[",
                pos_to_str(pos)
            )?;
            for dy in [-1, 0, 1] {
                for dx in [-1, 0, 1] {
                    if (dx, dy) == (0, 0) {
                        continue;
                    }
                    let points =
                        iter::successors(Some((x, y)), move |(x, y)| Some((x + dx, y + dy)))
                            .skip(1)
                            .take_while(|(x, y)| {
                                (0..BOARD_SIZE).contains(x) && (0..BOARD_SIZE).contains(y)
                            })
                            .map(|p| format!("Pos::{}", pos_to_str(p)))
                            .collect::<Vec<_>>();
                    if points.len() < 2 {
                        continue;
                    }
                    writeln!(
                        writer,
                        "  &[{}], // ({:2}, {:2})",
                        points.join(", "),
                        dx,
                        dy
                    )?;
                }
            }
            writeln!(writer, "];")?;
        }
    }

    writeln!(
        writer,
        "fn flip_lines(p: Pos) -> &'static [&'static [Pos]] {{"
    )?;
    writeln!(writer, "    match p {{")?;
    for x in 0..BOARD_SIZE {
        for y in 0..BOARD_SIZE {
            let pos = pos_to_str((x, y));
            writeln!(writer, "      Pos::{} => FLIP_LINE_{},", pos, pos)?;
        }
    }
    writeln!(writer, "    _ => unreachable!()")?;
    writeln!(writer, "    }}")?;
    writeln!(writer, "}}")?;

    Ok(())
}
