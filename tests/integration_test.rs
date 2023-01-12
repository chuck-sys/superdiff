use superdiff::cli::{Cli, ReportingMode};
use superdiff::comp::get_all_matches;
use superdiff::types::JsonRoot;

use std::fs::read_to_string;
use std::path::PathBuf;

macro_rules! vec_pathbuf {
    ( $( $x:expr ),* ) => {
        {
            vec![
                $(
                    PathBuf::from($x),
                )*
            ]
        }
    };
}

fn terraria_clone_files() -> Vec<PathBuf> {
    vec_pathbuf![
        "examples/TerrariaClone/src/Chunk.java",
        "examples/TerrariaClone/src/DoubleContainer.java",
        "examples/TerrariaClone/src/Entity.java",
        "examples/TerrariaClone/src/Inventory.java",
        "examples/TerrariaClone/src/ItemCollection.java",
        "examples/TerrariaClone/src/LightConverter.java",
        "examples/TerrariaClone/src/PerlinNoise.java",
        "examples/TerrariaClone/src/Player.java",
        "examples/TerrariaClone/src/RandConverter.java",
        "examples/TerrariaClone/src/TerrariaClone.java",
        "examples/TerrariaClone/src/TextField.java",
        "examples/TerrariaClone/src/World.java",
        "examples/TerrariaClone/src/WorldContainer.java"
    ]
}

fn similar_matches_files() -> Vec<PathBuf> {
    vec_pathbuf![
        "examples/similar-matches-in-same-group/file1.txt",
        "examples/similar-matches-in-same-group/file3.txt",
        "examples/similar-matches-in-same-group/file2.txt"
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
    let expected: JsonRoot =
        serde_json::from_str(&read_to_string("tests/expected/terraria_clone_eq_b20.json").unwrap())
            .unwrap();

    assert_eq!(matches, expected);
}

#[test]
fn it_puts_all_matches_in_same_group() {
    let args = Cli {
        lev_threshold: 5,
        line_threshold: 10,
        block_threshold: 5,
        verbose: false,
        files: similar_matches_files(),
        reporting_mode: ReportingMode::Json,
    };

    let matches = JsonRoot::from(get_all_matches(&args));
    let expected: JsonRoot = serde_json::from_str(
        &read_to_string("tests/expected/similar_matches_in_1_group.json").unwrap(),
    )
    .unwrap();

    assert_eq!(matches, expected);
}

#[test]
fn it_could_probably_check_stdin() {
    let args = Cli {
        lev_threshold: 0,
        line_threshold: 1,
        block_threshold: 20,
        verbose: true,
        files: vec![],
        reporting_mode: ReportingMode::Json,
    };

    assert!(args.files_from_stdin());
    assert_eq!(args.files.len(), 0);
}
