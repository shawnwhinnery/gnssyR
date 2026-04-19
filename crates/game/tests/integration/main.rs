/// Headless integration tests.
///
/// Use `SoftwareDriver` + `SimulatedBackend` — no GPU or display required.
mod scenes;

use game::scenes::Scene;
use gfx::driver::GraphicsDriver;
use gfx_software::SoftwareDriver;
use scenes::gfx_showcase::GfxShowcaseScene;

// ---------------------------------------------------------------------------
// Scene snapshot
// ---------------------------------------------------------------------------

/// Resolution used for all snapshot renders.  Small enough to be fast, large
/// enough to catch meaningful regressions.
const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

fn render_scene() -> Vec<u32> {
    let mut driver = SoftwareDriver::headless(WIDTH, HEIGHT);
    let scene = GfxShowcaseScene;
    driver.begin_frame();
    scene.draw(&mut driver);
    driver.end_frame();
    driver.pixels().to_vec()
}

fn snapshot_path() -> std::path::PathBuf {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots");
    std::fs::create_dir_all(&dir).expect("create snapshots dir");
    dir.join("gfx_scene.bin")
}

/// Encode pixel data as little-endian bytes for storage.
fn pixels_to_bytes(pixels: &[u32]) -> Vec<u8> {
    pixels.iter().flat_map(|p| p.to_le_bytes()).collect()
}

/// Render the GFX showcase scene and compare against a stored golden snapshot.
///
/// On first run (or when `UPDATE_SNAPSHOTS=1` is set) the snapshot is written
/// rather than compared — commit the resulting file to lock in the baseline.
#[test]
fn gfx_scene_snapshot() {
    let pixels = render_scene();
    let bytes = pixels_to_bytes(&pixels);
    let path = snapshot_path();

    let update = std::env::var("UPDATE_SNAPSHOTS").is_ok();

    if update || !path.exists() {
        std::fs::write(&path, &bytes).expect("failed to write snapshot");
        if update {
            println!("snapshot updated: {}", path.display());
        } else {
            println!("snapshot created: {}", path.display());
        }
        return;
    }

    let expected = std::fs::read(&path).expect("failed to read snapshot");
    if bytes != expected {
        // Write the actual output beside the golden file so it can be
        // inspected and, if correct, promoted with UPDATE_SNAPSHOTS=1.
        let actual_path = path.with_extension("actual.bin");
        let _ = std::fs::write(&actual_path, &bytes);
        panic!(
            "scene snapshot mismatch ({WIDTH}x{HEIGHT})\n\
             golden: {}\n\
             actual: {}\n\
             Re-run with UPDATE_SNAPSHOTS=1 to accept new output.",
            path.display(),
            actual_path.display(),
        );
    }
}
