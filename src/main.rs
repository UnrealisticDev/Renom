mod changes;
mod changesets;
mod engine;
mod logger;
mod pathfinder;
mod renamer;

use logger::Log;
use renamer::Renamer;

use std::{
    io::{stdin, Read},
    path::PathBuf,
};

fn main() {
    let root = PathBuf::from("test/TestEngineVersion4");
    let changeset = changesets::generate_code_changeset(
        "TestEngineVersion4",
        "TestEngineVersion4Post",
        "test/TestEngineVersion4",
        vec![PathBuf::from(
            "Source/TestEngineVersion4/TestEngineVersion4GameModeBase.h",
        )],
        vec![],
    );
    std::fs::create_dir_all(root.join(".renom/backup")).unwrap();
    engine::execute(changeset, root.join(".renom/backup"));
    return;

    let mut renamer = Renamer::new();
    if let Err(e) = renamer.start() {
        Log::error(format!("Project rename failed: {}", e));
    }

    Log::prompt("Press Enter to exit.");
    let _ = stdin().read(&mut [0u8]);
}
