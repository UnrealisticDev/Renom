mod pathfinder;
mod printer;
mod renamer;

use printer::Print;
use renamer::Renamer;

use std::io::{stdin, Read};

fn main() {
    let mut renamer = Renamer::new();
    if let Err(e) = renamer.start() {
        Print::error(format!("Project rename failed: {}", e));
    }

    Print::prompt("Press Enter to exit.");
    let _ = stdin().read(&mut [0u8]);
}
