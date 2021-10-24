mod pathfinder;
mod logger;
mod renamer;

use logger::Log;
use renamer::Renamer;

use std::io::{stdin, Read};

fn main() {
    let mut renamer = Renamer::new();
    if let Err(e) = renamer.start() {
        Log::error(format!("Project rename failed: {}", e));
    }

    Log::prompt("Press Enter to exit.");
    let _ = stdin().read(&mut [0u8]);
}
