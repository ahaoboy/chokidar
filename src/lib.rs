use clap::Parser;
use clap::ValueEnum;
use clean_path::clean;
use notify::Config;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify::Watcher;
use std::fmt::Display;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;
use wax::Glob;
extern crate colored; // not needed in Rust 2018+

use colored::*;

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
pub enum Shell {
    #[default]
    Bash,

    Zsh,
    Sh,
    Fish,
    Cmd,
    Powershell,
    Pwsh,
}

impl<'a> From<&'a str> for Shell {
    fn from(value: &'a str) -> Self {
        match value {
            "bash" => Shell::Bash,
            "zsh" => Shell::Zsh,
            "sh" => Shell::Sh,
            "fish" => Shell::Fish,
            "cmd" => Shell::Cmd,
            "powershell" => Shell::Powershell,
            "pwsh" => Shell::Pwsh,
            _ => Shell::Bash,
        }
    }
}

impl Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Sh => "sh",
            Shell::Fish => "fish",
            Shell::Cmd => "cmd",
            Shell::Powershell => "powershell",
            Shell::Pwsh => "pwsh",
        })
    }
}

fn time_to_str(s: u64) -> String {
    match s {
        0 => "immediately".to_string(),
        1..=59 => format!("in {s}s"),
        _ => {
            let m = s / 60;
            let s = s % 60;

            match s {
                0 => format!("in {m}m"),
                _ => format!("in {m}m {s}s"),
            }
        }
    }
}

#[derive(Parser, Debug, Clone, Default)]
#[command(version, about, long_about = None)]
pub struct Args {
    // debounce
    #[arg(short, long, default_value_t = 400)]
    debounce: usize,

    // #[arg(short, long, default_value_t = 0)]
    // throttle: usize,

    // #[arg(short, long, default_value_t = false)]
    // follow_symlinks: bool,

    // #[arg(short, long, default_value_t = Vec::new())]
    // ignore: Vec<String>,
    // ignore: null,
    // polling: false,
    // pollInterval: 100,
    // pollIntervalBinary: 300,
    // #[arg(short, long, default_value_t = false)]
    // verbose: bool,
    // #[arg(short, long, default_value_t = false)]
    // silent: bool,

    // initial
    #[arg(long, default_value_t = false)]
    initial: bool,

    // cwd
    #[arg(long)]
    cwd: Option<String>,

    // shell
    #[arg(long, default_value_t = Shell::Bash)]
    shell: Shell,

    // cmd
    #[arg(short, long)]
    cmd: String,

    // pattern
    #[clap()]
    pattern: String,
}

fn exec(shell: Shell, cmd: &str, cwd: String) {
    let now = Instant::now();
    println!("{}", (&format!("[Running({}): {}]", shell, cmd)).green());

    let op = match shell {
        Shell::Cmd => "/c",
        _ => "-c",
    };

    let mut c = Command::new(shell.to_string());

    c.args([op, cmd])
        .current_dir(&cwd)
        .stdout(Stdio::inherit())
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit());

    c.output().expect("command exec error");
    let time = time_to_str(now.elapsed().as_secs());
    println!("{}", format!("[Command was successful {time}]").green());
}

impl Args {
    pub fn new(
        debounce: usize,
        initial: bool,
        cwd: Option<String>,
        shell: Shell,
        cmd: String,
        pattern: String,
    ) -> Self {
        Self {
            debounce,
            initial,
            cwd,
            shell,
            cmd,
            pattern,
        }
    }
}

pub fn chokidar(args: Args) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    let pattern = &args.pattern;
    let cwd = args.cwd.unwrap_or(
        std::env::current_dir()
            .expect("can't get current_dir")
            .to_string_lossy()
            .to_string(),
    );
    let (prefix, glob) = Glob::new(pattern).unwrap().partition();
    let mut cwd_glob = PathBuf::from(cwd.clone());
    let prefix = dunce::canonicalize(cwd_glob.clone().join(prefix)).unwrap();
    let mut file_count: u32 = 0;
    cwd_glob.push(prefix);
    let cwd_glob = clean(cwd_glob);

    for entry in glob.walk(cwd_glob).filter_map(|i| i.ok()) {
        let path = entry.path();
        watcher
            .watch(path, RecursiveMode::Recursive)
            .unwrap_or_else(|_| panic!("watch file error: {:?} ", path));
        file_count += 1;
    }

    let shell = args.shell;

    if args.initial {
        println!("{}", ("[initial run]").green());
        exec(shell, &args.cmd, cwd.clone());
    }
    let debounce_fn = fns::debounce(
        move |_| exec(shell, &args.cmd, cwd.clone()),
        Duration::from_millis(args.debounce.try_into().unwrap()),
    );

    let file_str = match file_count {
        0..=999 => (&format!("{file_count}")).green(),
        1000..=4999 => (&format!("{file_count}")).yellow(),
        _ => (&format!("{file_count}")).red(),
    };

    println!(
        "{}",
        (&format!("[watching({file_str}): {pattern}]")).green()
    );

    for result in rx {
        match result {
            Ok(_) => {
                debounce_fn.call(());
            }
            Err(error) => println!("Error {error:?}"),
        }
    }
}
