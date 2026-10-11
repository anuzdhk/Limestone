//! Renders every factory preset onto one image, grouped by collection.
//!
//! cargo run --release -p adjust-core --example contact_sheet -- <input> <output.png>

use adjust_core::{get_collections, get_factory_presets, process_image, RgbImageF32};
use std::path::PathBuf;

const COLS: usize = 8;
const THUMB: usize = 240;
const GAP: usize = 6;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let input = PathBuf::from(args.next().ok_or("missing <input>")?);
    let output = PathBuf::from(args.next().ok_or("missing <output.png>")?);

    let source = RgbImageF32::open(&input)?.downsample(THUMB);
    let presets = get_factory_presets();
    let collections = get_collections();

    let rows: usize = collections.iter().map(|c| c.presets.len().div_ceil(COLS)).sum();
    let (cell_w, cell_h) = (source.width + GAP, source.height + GAP);
    let gap_rows = collections.len() * 4 * GAP;
    let mut sheet = RgbImageF32::new(COLS * cell_w, rows * cell_h + gap_rows);

    let mut y0 = 0;
    for col in &collections {
        println!("== {} ==", col.name);
        for (i, name) in col.presets.iter().enumerate() {
            let preset = presets.iter().find(|p| p.name == *name).unwrap();
            let out = process_image(&source, &preset.params);
            let (cx, cy) = ((i % COLS) * cell_w, y0 + (i / COLS) * cell_h);
            println!("  [{:>2}] row {} col {}: {}", i, i / COLS, i % COLS, name);
            for y in 0..out.height {
                for x in 0..out.width {
                    let (s, d) = (y * out.width + x, (cy + y) * sheet.width + cx + x);
                    sheet.r[d] = out.r[s];
                    sheet.g[d] = out.g[s];
                    sheet.b[d] = out.b[s];
                }
            }
        }
        y0 += col.presets.len().div_ceil(COLS) * cell_h + 4 * GAP;
    }
    sheet.save(&output)?;
    println!("wrote {}", output.display());
    Ok(())
}
