use std::path::PathBuf;
use std::process::Command;

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_adjust-cli"))
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("limestone_cli_{}_{}", name, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_test_png(path: &PathBuf, w: u32, h: u32) {
    let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x * 4) as u8, (y * 4) as u8, 128]));
    img.save(path).unwrap();
}

#[test]
fn list_presets_succeeds() {
    let out = cli().arg("list-presets").output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("Spicify"));
}

#[test]
fn processes_image_with_factory_preset() {
    let dir = temp_dir("factory");
    let input = dir.join("in.png");
    let output = dir.join("out.png");
    write_test_png(&input, 48, 32);

    let status = cli()
        .args(["--input", input.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .args(["--preset", "dramatic"])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(image::open(&output).unwrap().into_rgb8().dimensions(), (48, 32));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn processes_image_with_partial_preset_file() {
    let dir = temp_dir("custom");
    let input = dir.join("in.png");
    let output = dir.join("out.jpg");
    let preset = dir.join("preset.json");
    write_test_png(&input, 16, 16);
    std::fs::write(
        &preset,
        r#"{"name":"Mini","category":"Custom","description":"","params":{"color":{"saturation":0.5}}}"#,
    )
    .unwrap();

    let status = cli()
        .args(["--input", input.to_str().unwrap(), "--output", output.to_str().unwrap()])
        .args(["--preset-file", preset.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(output.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn unknown_preset_fails_cleanly() {
    let dir = temp_dir("unknown");
    let input = dir.join("in.png");
    write_test_png(&input, 8, 8);
    let out = cli()
        .args(["--input", input.to_str().unwrap(), "--preset", "does-not-exist"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let _ = std::fs::remove_dir_all(&dir);
}
