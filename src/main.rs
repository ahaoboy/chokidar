use chokidar::{chokidar, Args};
use clap::Parser;

fn main() {
    let args = Args::parse();
    chokidar(args);
}

// use dunce; // Avoids UNC paths on Windows.
// use std::path::Path;
// use wax::{Glob, Pattern};

// fn main() {
//     let path = Path::new("C:/wt/chokidar/src/main.rs");
//     let directory = Path::new(".");
//     let (prefix, glob) = Glob::new("../chokidar/src/**").unwrap().partition();

//     glob.walk_with_behavior(directory, || {
//         let prefix = dunce::canonicalize(directory.join(&prefix)).unwrap();
//         if dunce::canonicalize(path)
//             .unwrap()
//             .strip_prefix(&prefix)
//             .map(|path| glob.is_match(path))
//             .unwrap_or(false)
//         {
//             println!("{}", "aaa");
//         }
//     });

//     println!("{:?}", prefix);
// }
