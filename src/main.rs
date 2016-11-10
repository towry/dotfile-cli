extern crate getopts;
#[macro_use]
extern crate json;

use std::process::exit;
use std::fmt;
use std::env;
use std::path;
use std::io::prelude::*;
use std::io;
use std::fs;
use getopts::Options;

const APP_NAME: &'static str = "dotfile";
const DOT_FILE_DIR: &'static str = "dotfiles_test";
const DOT_FILE_DIRS: [&'static str; 3] = ["link", "backup", "source",];

struct Config<'a> {
    app_root_dir: path::PathBuf,
    input: &'a str,
    mapping: Option<json::JsonValue>
}

impl<'a> fmt::Display for Config<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.app_root_dir.display())
    }
}

impl<'a> Config<'a> {
    #[allow(dead_code)]
    fn mapping_has(&self, key: &str) -> bool {
        if self.mapping.is_none() {
            panic!("mapping file not parsed");
        }

        match self.mapping {
            Some(ref map) => {
                return !map[key].is_null();
            },
            None => {
                return false
            }
        }

        false
    }

    fn mapping_get(&self, key: &str) -> json::JsonValue {
        if self.mapping.is_none() {
            panic!("mapping file not parsed");
        }

        match self.mapping {
            Some(ref map) => {
                return map[key].clone();
            },
            None => {
                return json::JsonValue::new_object();
            }
        }
    }

    fn mapping_init(&mut self, file: &String) -> Result<(), (io::Error)> {
        let mut buffer = String::new();
        let mut f = try!(fs::File::open(file));

        try!(f.read_to_string(&mut buffer));

        let obj = json::parse(&*buffer);
        if obj.is_ok() {
            self.mapping = obj.ok();
        } else {
            self.mapping = Some(json::JsonValue::new_object());
            return Err(io::Error::new(io::ErrorKind::Other, "error"));
        }

        Ok(())
    }
}

fn execute(config: &mut Config, out: Option<String>) {
    initial_check(config);

    match config.input {
        "a" => {
            link_add(config, out);
        },
        "r" => {
            link_remove(config, out);
        },
        _ => {
            return;
        }
    };
}

fn link_add(config: &Config, out: Option<String>) {
    let base_dir = config.app_root_dir.join("link/");
    let file = out.unwrap();
    let file_path = path::Path::new(&file);

    let file_to_add = path::PathBuf::from(homedir(&file_path).unwrap());
    if !file_to_add.exists() {
        println!("{} not exist", file_to_add.display());
        return quit();
    }

    if is_symlink(&file_to_add) {
        println!("{} is a symlink", file_to_add.display());
    } else {
        println!("not a symlink");
    }
}

fn link_remove(config: &Config, out: Option<String>) {
    println!("link remove");
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn homedir(file: &path::Path) -> Option<path::PathBuf> {
    let home = env::home_dir();
    home.map(|dir| { dir.join(file) })
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

    let output: Option<String>;
    let input: &str;
    if matches.opt_present("a") {
        input = "a";
        output = matches.opt_str("a");
    } else if matches.opt_present("r") {
        input = "r";
        output = matches.opt_str("r");
    } else {
        print_usage(APP_NAME, opts);
        return;
    }

    if !output.is_some() {
        print_usage(APP_NAME, opts);
        return;
    }

    let root_dir = env::home_dir();
    if !root_dir.is_some() {
        panic!("dotfile couldn't find your home directory. \
            This probably means that $HOME was not set.");
    }
    let root_dir_path = root_dir.map(|dir| { dir.join(DOT_FILE_DIR) });

    let mut config = Config {
        app_root_dir: root_dir_path.unwrap(),
        input: input,
        mapping: None,
    };

    // consume the output.
    execute(&mut config, output);
}

fn initial_check(config: &mut Config) {
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

       if prompt("mapping.json file not exist.\n Do you want to create it from \
    current directory?") {
           create_mapping_file(config);
           // TODO: init mapping.json from current directory.
       }
   }

   let mapping_name = String::from(config.app_root_dir.join("mapping.json").to_str().unwrap());
   config.mapping_init(&mapping_name).ok();
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

fn is_symlink(file: &path::Path) -> bool {
    let metadata = file.symlink_metadata();
    match metadata {
        Ok(meta) => {
            return meta.file_type().is_symlink();
        },
        Err(err) => {
            panic!(err);
            // return false;
        }
    }
}
