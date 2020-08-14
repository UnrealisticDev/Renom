mod pathfinder;
mod printer;
mod renamer;

use printer::Print;
use renamer::Renamer;

fn main() {
    let mut renamer = Renamer::new();
    if let Err(e) = renamer.start() {
        Print::error(format!("Project rename failed: {}", e));
    }
}
