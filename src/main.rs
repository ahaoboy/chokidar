use chokidar::{chokidar, Args};
use clap::Parser;

fn main() {
    let args = Args::parse();
    chokidar(args);
}
