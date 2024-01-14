use getopts::Options;
use osd::{download_link, download_save_sub, login, search_for_subtitle_id_key, Movie, Url};
use serde::{Deserialize, Serialize};
use std::{
    env,
    error::Error,
    process::{self},
};

//fn _type_of<T>(_: &T) {
//    println!("{}", std::any::type_name::<T>())
//}

#[derive(Serialize, Deserialize)]
struct Config {
    key: String,
    user: String,
    password: String,
    language: String,
    useragent: String,
}

//Manages osd.conf file
impl Config {
    fn new(
        key: String,
        user: String,
        password: String,
        language: String,
        useragent: String,
    ) -> Self {
        Self {
            key,
            user,
            password,
            language,
            useragent,
        }
    }

    //Creates new config osd.conf file
    fn build() -> Result<Config, Box<dyn Error>> {
        let config = Config::new(
            "".to_owned(),
            "".to_owned(),
            "".to_owned(),
            "en".to_owned(),
            "Opensubtitles downloader".to_owned(),
        );
        Config::write_config(&config)?;
        Ok(config)
    }

    //Loads config file or creates a new one
    fn load_config() -> Result<Config, Box<dyn Error>> {
        let home_dir = std::env::var_os("HOME").ok_or("No home directory")?;
        let mut config_path = std::path::PathBuf::new();

        config_path = config_path.join(home_dir).join(".config").join("osd");
        std::fs::create_dir_all(config_path.clone())?;

        config_path = config_path.join("osd.conf");

        //Checking if osd.conf exists
        let config = if let Ok(content) = std::fs::read(&config_path) {
            //from toml to object
            let config: Config = match toml::from_str(&String::from_utf8(content)?) {
                Ok(config) => config,
                Err(e) => Err(format!("Error reading config file: {}", e.message()))?,
            };

            // if config.user.is_empty()
            //     || config.key.is_empty()
            //     || config.password.is_empty()
            //     || config.language.is_empty()
            //     || config.useragent.is_empty()
            // {
            //     return Err("Config file osd.conf fields cannot be empty.")?;
            // }
            config
        } else {
            //Create a new one.
            Config::build()?
        };

        Ok(config)
    }

    //Write a osd.conf file
    fn write_config(config: &Config) -> Result<(), Box<dyn Error>> {
        let home_dir =
            std::env::var_os("HOME").ok_or("No home environtment variable found. Strange")?;
        let mut config_path = std::path::PathBuf::new();

        config_path = config_path.join(home_dir).join(".config").join("osd");
        std::fs::create_dir_all(config_path.clone())?;
        config_path = config_path.join("osd.conf");

        let config_string = toml::to_string(config)?;
        std::fs::write(&config_path, config_string)?;

        Ok(())
    }
}

///Parsed arguments properties
struct ParsedArgs {
    use_gui: bool,
    gui_mode: String,
    path: String,
    verbose: bool,
}

impl ParsedArgs {
    fn new(use_gui: bool, gui_mode: String, path: String, verbose: bool) -> Self {
        Self {
            use_gui,
            gui_mode,
            path,
            verbose,
        }
    }

    /// Builds a struct of arguments
    fn build() -> ParsedArgs {
        // -- parse arguments
        let mut opts = Options::new();
        opts.optflag("g", "gui", "Choose subtitle from a dialog list");
        opts.optflag("h", "help", "Prints this help");
        opts.optflag("v", "verbose", "Prints verbose information");
        opts.optflag("V", "version", "Prints version information");

        let args: Vec<String> =env::args().collect();
        //Checks for unrecognized options
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("{}", f);
                print_help(opts);
                std::process::exit(1);
            }
        };

        //prints help and exits
        if matches.opt_present("h") {
            print_help(opts);
            std::process::exit(0);
        };

        //prints osd current version
        if matches.opt_present("V") {
            println!(env!("CARGO_PKG_VERSION"));
            std::process::exit(0)
        }

        let verbose = matches.opt_present("v");

        let mut use_gui = false;
        let mut gui_mode = Default::default();

        if matches.opt_present("g") {
            use_gui = true;

            //Detect desktop mode, default gtk
            let current_desktop = match env::var_os("XDG_CURRENT_DESKTOP") {
                Some(desktop) => desktop.into_string().unwrap_or("gtk".to_owned()),
                None => "gtk".to_string(),
            };

            gui_mode = if [
                "Cinnamon", "GNOME", "XFCE", "xfce4", "bspwm", "gnome", "gtk",
            ]
            .contains(&current_desktop.as_str())
            {
                "gtk".to_string()
            } else {
                "qt".to_string()
            };
        }

        let free_args = matches.free.len();
        //Only accepts one argument, the movie filename
        if free_args != 1 {
            print_help(opts);
            std::process::exit(0);
        }

        //Returns struct of ParsedArgs
        ParsedArgs::new(
            use_gui,
            gui_mode,
            matches.free.first().unwrap().to_string(),
            verbose,
        )
    }
}

/// prints help
fn print_help(opts: Options) {
    let brief = "usage: osd [-h] [-g] [-v] movie";
    println!("{}", opts.usage(brief));
}

fn run(parsed_args: ParsedArgs, config: Config) -> Result<(), Box<dyn Error>> {
    //Gets movie properties
    let movie = Movie::build(&parsed_args.path)?;

    if parsed_args.verbose {
        println!("Using api key: {}", config.key);
    }

    let file_id = search_for_subtitle_id_key(
        &movie.title,
        &movie.hash,
        &config.key,
        &config.language,
        parsed_args.use_gui,
        &parsed_args.gui_mode,
        &config.useragent,
    )?;

    let token = login(
        &config.key,
        &config.user,
        &config.password,
        &config.useragent,
    )?;

    if parsed_args.verbose {
        println!("Login token: {}", token);
    }

    // Downloadss suitable url subtitle
    let url: Url = download_link(&file_id, &token, &config.key, &config.useragent)?;

    if parsed_args.verbose {
        println!("Subtitle to be downloaded: {}", url.link);
        println!("Remaining requests for the day: {}", url.remaining);
        println!("Requests reset time: {}", url.reset_time);
    }

    //Downloads the subtitle and saves on the correct path
    download_save_sub(&url.link, &movie.path)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    //First loading config, because parseargs calls process::exit()
    let config = Config::load_config()?;
    let parsed_args = ParsedArgs::build();

    match run(parsed_args, config) {
        Ok(_) => Ok(()),
        Err(e) => {
            if let Some(err) = e.downcast_ref::<reqwest::Error>() {
                eprintln!("Request Error: {}", err);
            } else {
                eprintln!("{}", e);
            }
            process::exit(1);
        }
    }
}
