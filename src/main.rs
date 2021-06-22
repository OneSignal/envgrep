use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

use glob::glob;
use regex::bytes::{Regex, RegexBuilder};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Search through the environment variables of all running processes on the
/// system and report on all variables that match the specified pattern.
struct Options {
    /// Print all error messages as they occur instead of hiding them
    #[structopt(short, long)]
    verbose: bool,

    /// Perform case-insensitive matching with the specified regex
    #[structopt(short = "i", long)]
    case_insensitive: bool,

    /// Regex pattern to use to search for environment variables. Matches on
    /// both parts of the `KEY=value` string (independently), so parts of the
    /// environment variable name, value, or both can be used here.
    #[structopt(name = "PATTERN")]
    pattern: String,
}

struct EnvVariable {
    key: String,
    value: String,
}

struct Process {
    cmdline: String,
    variables: Vec<EnvVariable>,
}

fn main() {
    let opt = Options::from_args();

    let r = RegexBuilder::new(&opt.pattern[..])
        .case_insensitive(opt.case_insensitive)
        .build()
        .unwrap();

    let stdout = std::io::stdout();
    let mut lock = stdout.lock();

    for entry in glob("/proc/*/environ").unwrap() {
        let path = entry.unwrap();

        match grep_file(&path, &r) {
            Ok(process) => {
                if process.variables.is_empty() {
                    continue;
                }

                lock.write_fmt(format_args!("{} ({}):\n", path.display(), process.cmdline))
                    .unwrap();
                for var in process.variables {
                    lock.write_fmt(format_args!("{} = {:?}\n", var.key, var.value))
                        .unwrap();
                }

                lock.write(b"\n").unwrap();
            }
            Err(e) => {
                if opt.verbose {
                    eprintln!("Error reading {} - {}", path.display(), e);
                }
            }
        }
    }
}

fn load_cmdline(environfile_path: &Path) -> Result<String, Box<dyn Error>> {
    let parent = environfile_path.parent().unwrap().join("cmdline");
    let reader = BufReader::new(File::open(&parent)?);

    let mut cmdline = String::new();

    let mut add_space = false;

    for group in reader.split(0) {
        let group = group?;
        if group.is_empty() {
            continue;
        }

        if add_space {
            cmdline.push(' ');
        } else {
            add_space = true;
        }

        let slice = std::str::from_utf8(&group[..])?;
        cmdline.push_str(slice);
    }

    Ok(cmdline)
}

fn grep_file(path: &Path, regex: &Regex) -> Result<Process, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let cmdline = load_cmdline(path)?;

    let mut variables = Vec::new();

    for group in reader.split(0) {
        let group = group?;
        if group.is_empty() {
            continue;
        }

        if regex.is_match(&group) {
            let slice = std::str::from_utf8(&group[..])?;
            let idx = slice.find('=').unwrap();

            let key = slice[..idx].to_owned();
            let value = slice[(idx + 1)..].to_owned();

            variables.push(EnvVariable { key, value });
        }
    }

    Ok(Process { variables, cmdline })
}
