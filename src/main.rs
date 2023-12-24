use getopts::Options;
use osd::download_save_file;
use osd::download_url;
use osd::login;
use osd::search_for_subtitle_id_key;
use osd::Movie;
use osd::Url;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::error::Error;
use std::process;

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
    fn build() -> Result<Config, Box<dyn Error>> {
        let config = Config {
            key: "".to_owned(),
            user: "".to_owned(),
            password: "".to_owned(),
            language: "en".to_owned(),
            useragent: "Opensubtitles Downloader".to_owned(),
        };
        Config::write_config(&config)?;
        Ok(config)
    }

    //Loads config file or creates a new one
    fn load_config() -> Result<Config, Box<dyn Error>> {
        let home_dir = std::env::var_os("HOME").ok_or("No home directory")?;
        let mut config_path = std::path::PathBuf::new();
        config_path = config_path
            .join(home_dir.clone())
            .join(".config")
            .join("osd");
        std::fs::create_dir_all(config_path.clone())?;
        config_path = config_path.join("osd.conf");

        //Checking if osd.conf exists
        let config = if let Ok(content) = std::fs::read(&config_path) {
            let config: Config = toml::from_str(&String::from_utf8(content)?)?;

            if config.user.is_empty()
                || config.key.is_empty()
                || config.password.is_empty()
                || config.language.is_empty()
                || config.useragent.is_empty()
            {
                return Err("Config file osd.conf fields cannot be empty.")?;
            }
            config
        } else {
            //if config file not found create a new one.
            Config::build()?
        };

        Ok(config)
    }

    //Write a osd.conf file
    fn write_config(config: &Config) -> Result<(), Box<dyn Error>> {
        let home_dir = std::env::var_os("HOME").ok_or("No home directory")?;
        let mut config_path = std::path::PathBuf::new();
        config_path = config_path
            .join(home_dir.clone())
            .join(".config")
            .join("osd");
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
    /// Builds a struct of arguments
    fn build(args: &[String]) -> ParsedArgs {
        // -- parse arguments
        let mut opts = Options::new();
        opts.optflag("g", "gui", "Choose subtitle from a dialog list");
        opts.optflag("h", "help", "Prints this help");
        opts.optflag("v", "verbose", "Prints more information");

        //Checks for unrecognized options
        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("{}", f);
                print_help(opts);
                std::process::exit(0);
            }
        };

        //prints help and exits
        if matches.opt_present("h") {
            print_help(opts);
            std::process::exit(1);
        };

        let verbose = matches.opt_present("v");

        let mut use_gui = false;
        let mut gui_mode = Default::default();

        if matches.opt_present("g") {
            use_gui = true;

            //Detect desktop mode
            let desktop = match env::var_os("XDG_CURRENT_DESKTOP") {
                Some(desktop) => desktop.into_string().unwrap(),
                None => "gtk".to_string(),
            };

            gui_mode = if [
                "Cinnamon", "GNOME", "XFCE", "xfce4", "bspwm", "gnome", "gtk",
            ]
            .contains(&desktop.as_str())
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
        return ParsedArgs {
            use_gui,
            gui_mode,
            path: matches.free.first().unwrap().to_string(),
            verbose,
        };
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
    
    // download suitable subtitle
    let url: Url = download_url(&file_id, &token, &config.key, &config.useragent)?;

    if parsed_args.verbose {
        println!("Remaining requests for the day: {}", url.remaining);
        println!("Requests reset time: {}", url.reset_time);
    }

    download_save_file(&url.link, &movie.path)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    //First loading config, because parseargs calls process::exit()
    let config = Config::load_config()?;
    let parsed_args = ParsedArgs::build(&args);

    //parse arg to a convenient struct

    match run(parsed_args, config) {
        Ok(_) => process::exit(0),
        Err(e) => {
            if let Some(err) = e.downcast_ref::<reqwest::Error>() {
                eprintln!("Request Error: {}", err);
            } else {
                eprintln!("{}", e);
            }
        }
    }
    Ok(())
}
