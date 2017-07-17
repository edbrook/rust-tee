use std::env;

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};


#[derive(Debug, Hash, Eq, PartialEq)]
enum ConfigOption {
    Help,
    Append
}


#[derive(Debug)]
struct Config {
    options: HashSet<ConfigOption>,
    files: Vec<String>
}


impl Config {
    fn new() -> Config {
        Config {
            options: HashSet::new(),
            files: vec![]
        }
    }

    fn is_help_set(&self) -> bool {
        self.options.contains(&ConfigOption::Help)
    }

    fn is_append_set(&self) -> bool {
        self.options.contains(&ConfigOption::Append)
    }
}


fn tee<T: io::Read>(input: &mut T, files: &Vec<File>) -> io::Result<()> {
    let mut stdout = io::stdout();
    let line: &mut[u8] = &mut[0; 4096];
    loop {
        let n = input.read(line)?;
        if n>0 {
            stdout.write_all(&line[0..n])?;
            for mut file in files {
                file.write_all(&line[0..n])?;
            }
        } else {
            break;
        }
    }
    Ok(())
}


fn show_help() {
    println!("Usage: tee [-a] [-h|--help] [file ...]");
    println!("  -a          Append to any file listed rather than truncating");
    println!("  -h  --help  This help");
    println!("  <file>      Zero or more files to write the output to");
}


fn parse_args(args: &Vec<String>) -> Config {
    let mut config = Config::new();
    for arg in &args[1..args.len()] {
        if arg == "--help" || arg == "-h" {
            config.options.insert(ConfigOption::Help);
        } else if arg == "-a" {
            config.options.insert(ConfigOption::Append);
        } else if arg.starts_with("-") {
            writeln!(&mut io::stderr(), "Ignoring unknown option: {}", arg)
                .expect("Error while parsing options!");
        } else {
            config.files.push(arg.to_string());
        }
    }
    config
}


fn open_files(file_names: &Vec<String>, append: bool) -> io::Result<Vec<File>> {
    let mut files = vec![];
    for file_name in file_names {
        let mut open_opts = OpenOptions::new();
        open_opts.write(true)
            .create(true)
            .append(append)
            .truncate(!append);
        files.push(open_opts.open(file_name)?);
    }
    Ok(files)
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = parse_args(&args);
    
    if opts.is_help_set() {
        show_help();
        std::process::exit(0);
    }

    let files = match open_files(&opts.files, opts.is_append_set()) {
        Ok(f)   => f,
        Err(e)  => {
            println!("Error opening file(s): {}", e);
            std::process::exit(1);
        }
    };

    tee(&mut io::stdin(), &files)
        .expect("Error while writing output!");
}
