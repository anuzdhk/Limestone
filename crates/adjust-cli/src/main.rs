use adjust_core::{get_factory_presets, process_image, Preset, RgbImageF32};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "adjust-cli")]
#[command(about = "LimeStone CLI image adjustment engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Input image file path (PNG, JPG, TIFF, WebP, etc.)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output destination file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Factory preset name to apply (e.g. "Spicify", "Dramatic", "Photo Pop")
    #[arg(short, long, default_value = "Spicify")]
    preset: String,

    /// Optional path to custom JSON preset file
    #[arg(long)]
    preset_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all available built-in factory presets
    ListPresets,
    /// Export a factory preset configuration to a JSON file
    ExportPreset {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(cmd) = cli.command {
        match cmd {
            Commands::ListPresets => {
                println!("=== LimeStone Factory Presets ===");
                for preset in get_factory_presets() {
                    println!("\n* {} [{}]", preset.name, preset.category);
                    println!("  {}", preset.description);
                }
                return Ok(());
            }
            Commands::ExportPreset { name, output } => {
                let presets = get_factory_presets();
                if let Some(preset) = presets.iter().find(|p| p.name.eq_ignore_ascii_case(&name)) {
                    let json = preset.to_json()?;
                    std::fs::write(&output, json)?;
                    println!("Exported preset '{}' to {}", preset.name, output.display());
                    return Ok(());
                } else {
                    eprintln!("Error: Preset '{}' not found.", name);
                    std::process::exit(1);
                }
            }
        }
    }

    // Processing mode
    let input_path = match cli.input {
        Some(p) => p,
        None => {
            eprintln!("Error: --input <path> is required. Run with --help for instructions.");
            std::process::exit(1);
        }
    };

    let output_path = match cli.output {
        Some(p) => p,
        None => {
            let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = input_path.extension().unwrap_or_default().to_string_lossy();
            PathBuf::from(format!("{}_{}.{}", stem, cli.preset.to_lowercase(), ext))
        }
    };

    println!("Loading image: {}", input_path.display());
    let load_start = Instant::now();
    let f_img = RgbImageF32::open(&input_path)?;
    let (width, height) = (f_img.width, f_img.height);
    println!("Image loaded: {}x{} in {:.2?}", width, height, load_start.elapsed());

    // Resolve parameters
    let params = if let Some(preset_path) = cli.preset_file {
        println!("Loading custom preset file: {}", preset_path.display());
        let content = std::fs::read_to_string(preset_path)?;
        let custom: Preset = serde_json::from_str(&content)?;
        custom.params
    } else {
        let presets = get_factory_presets();
        let selected = presets
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(&cli.preset))
            .ok_or_else(|| format!("Unknown preset '{}'. Use 'list-presets' to view available presets.", cli.preset))?;
        println!("Applying preset: {} ({})", selected.name, selected.category);
        selected.params.clone()
    };

    println!("Processing image through the adjustment pipeline...");
    let proc_start = Instant::now();
    let out_f_img = process_image(&f_img, &params);
    let proc_dur = proc_start.elapsed();
    println!("Pipeline completed in {:.2?} ({:.1} MP/s)", proc_dur, (width * height) as f64 / 1_000_000.0 / proc_dur.as_secs_f64());

    println!("Saving output to: {}", output_path.display());
    out_f_img.save(&output_path)?;
    println!("Successfully processed and saved.");

    Ok(())
}
