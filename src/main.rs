extern crate getopts;

use std::process::exit;
use std::fmt;
use std::env;
use std::path;
use std::io;
use std::fs;

use getopts::Options;

const APP_NAME: &'static str = "dotfile";
const DOT_FILE_DIR: &'static str = "dotfiles_test";
const DOT_FILE_DIRS: [&str; 3] = ["link", "backup", "source",];

struct Config<'a> {
    app_root_dir: path::PathBuf,
    input: &'a str,
}

impl<'a> fmt::Display for Config<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.app_root_dir.display())
    }
}

fn execute(config: &Config, out: &Vec<String>) {
    initial_check(config);

    let a = match config.input {
        "a" => {
            println!("got {}", config.input);
            config.input
        },
        "r" => {
            println!("got {}", config.input);
            config.input
        },
        _ => {
            return;
        }
    };
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();

    opts.optopt("a", "add", "add file to manage list", "FILE");
    opts.optopt("r", "remove", "remove file from manage list", "FILE");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(_) => { print_usage(APP_NAME, opts); exit(-1); }
    };

    if matches.opt_present("h") {
        print_usage(APP_NAME, opts);
        return;
    }

    let output: Vec<String>;
    let input: &str;
    if matches.opt_present("a") {
        input = "a";
        output = matches.opt_strs("a");
    } else if matches.opt_present("r") {
        input = "r";
        output = matches.opt_strs("r");
    } else {
        print_usage(APP_NAME, opts);
        return;
    }

    if output.is_empty() {
        print_usage(APP_NAME, opts);
        return;
    }

    let root_dir = env::home_dir();
    if !root_dir.is_some() {
        panic!("dotfile couldn't find your home directory. \
            This probably means that $HOME was not set.");
    }
    let root_dir_path = root_dir.map(|dir| { dir.join(DOT_FILE_DIR) });

    let config = Config { 
        app_root_dir: root_dir_path.unwrap(),
        input: input,
    };
    
    execute(&config, &output);
}

fn initial_check(config: &Config) {
   if !config.app_root_dir.is_dir() {
       println!(".dotfiles directory not exist.");
       if config.app_root_dir.is_file() {
           println!(".dotfiles file exist.");
           return quit();
       }
       fs::DirBuilder::new()
           .create(&config.app_root_dir).unwrap();
       println!("created .dotfiles directory");

       create_mapping_file(config);
       create_other_directory(config);
   } else if !config.app_root_dir.join("mapping.json").is_file() {
       // Scan the current dotfile directory, create mapping.json.
       // If the user prompted with yes, create the mapping.json file 
       // and initialize it with current exist files.
       // else just create a mapping.json file.

       if prompt("mapping.json file not exist.\n Do you want to create it from \
    current directory?") {
           create_mapping_file(config);
       } else {
           println!("...");
       }
   }
}

fn create_mapping_file(config: &Config) {
    let mapping = config.app_root_dir.join("mapping.json");
    match fs::File::create(mapping) {
        Ok(_) => {},
        Err(err) => { panic!(err) },
    }
    println!("created mapping.json file.");
}

fn create_other_directory(config: &Config) {
    let builder = fs::DirBuilder::new();
    for x in &DOT_FILE_DIRS {
        builder.create(config.app_root_dir.join(x)).unwrap();
    }
}

fn prompt<T>(question: T) -> bool
    where T: fmt::Display {
    println!("{}", question);
    
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(err) => { panic!(err) },
    }

    let input_str = input.trim();

    if input_str == "y" || input_str == "yes" {
        return true;
    }

    return false;
}

fn quit() {
    println!("quit");
    exit(1);
}
