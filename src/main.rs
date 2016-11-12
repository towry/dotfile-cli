extern crate getopts;

use std::process::exit;
use std::fmt;
use std::env;
use std::path;
use std::io;
use std::fs;
use getopts::Options;

#[macro_use]
mod macros;

const APP_NAME: &'static str = "dotfile";
const DOT_FILE_DIR: &'static str = "dotfiles_test";
const DOT_FILE_DIRS: [&'static str; 3] = ["link", "backup", "source",];

struct Config<'a> {
    app_root_dir: path::PathBuf,
    input: &'a str,
}

impl<'a> fmt::Display for Config<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.app_root_dir.display())
    }
}


fn execute(config: &mut Config, out: Option<String>) {
    initial_check(config);

    let result = match config.input {
        "a" => {
            link_add(config, out)
        },
        "r" => {
            link_remove(config, out)
        },
        _ => {
            return;
        }
    };

    if result.is_ok() {
        println!(":-) done.");
    }
}

fn link_add(config: &Config, out: Option<String>) -> Result<(), ()> {
    let base_dir = config.app_root_dir.join("link/");
    let file = out.unwrap();
    let file_path = path::Path::new(&file);

    let file_to_add_unresolved = path::PathBuf::from(homedir(&file_path).unwrap());

    if !file_to_add_unresolved.exists() {
        println!("{} not exist", file_to_add_unresolved.display());
        fail();
    }

    let file_to_add = file_to_add_unresolved.canonicalize().unwrap();

    match ensure_file_under_homedir(&file_to_add) {
        Ok(_) => {},
        Err(_) => {
            println!(":( you can only add file under home dir.");
            return Err(());
        }
    }

    if is_symlink(&file_to_add) {
        println!("{} is a symlink", file_to_add.display());
        fail();
    }

    if file_to_add.is_file() {
        // copy file.
        let mut path_buf = path::PathBuf::new();
        let home_dir = env::home_dir();
        if !home_dir.is_some() {
            panic!("{} could not access your home dir.", APP_NAME);
        }

        path_relative(&file_to_add, &home_dir.unwrap(), &mut path_buf);
        copy_file(&file_to_add, &base_dir.join(path_buf.as_path())).ok();
    } else if file_to_add.is_dir() {
        // copy dir.
        // TODO, canonicalize this.
        // let file_move_to = base_dir.join(&file);
        // If `file_to_add` is : a/b/c/, and base_dir is: e/f,
        // then the result will be e/f/c/.
        copy_dir(&file_to_add, &base_dir).ok();
    }

    Ok(())
}

fn link_remove(config: &Config, out: Option<String>) -> Result<(), ()> {
    println!("link remove");
    Ok(())
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
    };

    // consume the output.
    execute(&mut config, output);
}

fn initial_check(config: &mut Config) {
   if !config.app_root_dir.is_dir() {
       println!(".dotfiles directory not exist.");
       if config.app_root_dir.is_file() {
           println!(".dotfiles file exist.");
           return fail();
       }
       fs::DirBuilder::new()
           .create(&config.app_root_dir).unwrap();
       println!("created .dotfiles directory");

       create_other_directory(config);
   }
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

fn fail() -> ! {
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
        }
    }
}

fn copy_file(from: &path::Path, to: &path::Path) -> Result<(), (io::Error)> {
    // create directory first.
    let parent = to.parent().unwrap();
    try!(fs::create_dir_all(parent));

    debugln!("copying file {} - {}", from.display(), to.display());
    try!(fs::copy(from, to));
    Ok(())
}

fn copy_dir(from: &path::Path, to: &path::Path) -> Result<(), (io::Error)> {
    let parent = from.parent().unwrap();
    visit_dirs(from, &move |file: &fs::DirEntry| {
        let mut to_join_buf = path::PathBuf::new();
        path_relative(&file.path(), &parent, &mut to_join_buf);
        let to_join = to_join_buf.as_path();

        match copy_file(&file.path(), &to.join(&to_join)) {
            Ok(_) => { },
            Err(e) => {
                panic!(e);
            }
        };
    }).ok();

    Ok(())
}

fn visit_dirs(dir: &path::Path, cb: &Fn(&fs::DirEntry)) -> Result<(), (io::Error)> {
    if dir.is_dir() {
        for entry in try!(fs::read_dir(dir)) {
            let entry = try!(entry);
            let path = entry.path();
            if path.is_dir() {
                try!(visit_dirs(&path, cb));
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

// To make sure the file is under home dir.
fn ensure_file_under_homedir(p: &path::Path) -> Result<(), ()> {
    let home = env::home_dir();
    let home_dir = match home {
        Some(p) => { p },
        None => { panic!("{} could not access your home dir.", APP_NAME); },
    };

    if !p.starts_with(home_dir) {
        return Err(());
    }

    Ok(())
}


fn path_relative(file: &path::Path, prefix: &path::Path, buf: &mut path::PathBuf) {
    let file_arr: Vec<&std::ffi::OsStr> = file.iter().collect();
    let prefix_arr: Vec<&std::ffi::OsStr> = prefix.iter().collect();

    let relative: Vec<&std::ffi::OsStr> = file_arr[prefix_arr.len() ..].to_vec();

    for item in relative {
        buf.push(item);
    }
}
