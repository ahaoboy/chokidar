use clap::Parser;
use clap::ValueEnum;
use notify::RecursiveMode;
use notify::Watcher;
use notify_debouncer_full::new_debouncer;
use std::fmt::Display;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use wax::Glob;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Shell {
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

fn green(s: &str) {
    println!("\x1b[32m{}\x1b[0m", s);
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
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
    green(&format!("[Running({}): {}]", shell, cmd));

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
    green("[Command was successful]");
}

fn main() {
    let args = Args::parse();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(
        Duration::from_millis(args.debounce.try_into().unwrap()),
        None,
        tx,
    )
    .expect("debouncer create error");
    let watcher = debouncer.watcher();
    let pattern = &args.pattern;
    let cwd = args.cwd.unwrap_or(
        std::env::current_dir()
            .expect("can't get current_dir")
            .to_string_lossy()
            .to_string(),
    );
    let glob = Glob::new(pattern).unwrap();
    for entry in glob.walk(cwd.clone()).filter_map(|i| i.ok()) {
        let path = entry.path();
        watcher
            .watch(path, RecursiveMode::Recursive)
            .unwrap_or_else(|_| panic!("watch file error: {:?} ", path));
    }

    let shell = args.shell;

    if args.initial {
        green("[initial run]");
        exec(shell, &args.cmd, cwd.clone());
    }
    green(&format!("[watching: {}]", pattern));
    for result in rx {
        match result {
            Ok(_) => {
                // events.iter().for_each(|event| println!("Event {event:?}"));
                exec(shell, &args.cmd, cwd.clone());
            }
            Err(error) => println!("Error {error:?}"),
        }
    }
}
