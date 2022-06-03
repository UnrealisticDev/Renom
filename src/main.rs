mod changes;
mod changesets;
mod director;
mod engine;
mod logger;

use director::Director;

fn main() {
    Director::start_interactive_rename();
}
