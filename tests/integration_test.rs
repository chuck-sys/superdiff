use superdiff::cli::{Cli, ReportingMode};
use superdiff::types::JsonRoot;
use superdiff::comp::get_all_matches;

use std::path::PathBuf;
use std::fs::read_to_string;

fn terraria_clone_files() -> Vec<PathBuf> {
    vec![
        PathBuf::from("examples/TerrariaClone/src/Chunk.java"),
        PathBuf::from("examples/TerrariaClone/src/DoubleContainer.java"),
        PathBuf::from("examples/TerrariaClone/src/Entity.java"),
        PathBuf::from("examples/TerrariaClone/src/Inventory.java"),
        PathBuf::from("examples/TerrariaClone/src/ItemCollection.java"),
        PathBuf::from("examples/TerrariaClone/src/LightConverter.java"),
        PathBuf::from("examples/TerrariaClone/src/PerlinNoise.java"),
        PathBuf::from("examples/TerrariaClone/src/Player.java"),
        PathBuf::from("examples/TerrariaClone/src/RandConverter.java"),
        PathBuf::from("examples/TerrariaClone/src/TerrariaClone.java"),
        PathBuf::from("examples/TerrariaClone/src/TextField.java"),
        PathBuf::from("examples/TerrariaClone/src/World.java"),
        PathBuf::from("examples/TerrariaClone/src/WorldContainer.java"),
    ]
}

#[test]
fn it_outputs_correct_matches_for_terraria_clone() {
    let args = Cli {
        lev_threshold: 0,
        line_threshold: 1,
        block_threshold: 20,
        verbose: true,
        files: terraria_clone_files(),
        reporting_mode: ReportingMode::Json,
    };

    let matches = JsonRoot::from(get_all_matches(&args));
    let expected: JsonRoot = serde_json::from_str(&read_to_string("tests/expected/terraria_clone_eq_b20.json").unwrap()).unwrap();

    assert_eq!(matches, expected);
}
