use clap::{command, Arg, ArgAction};
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
    custom_title: String,
    verbose: bool,
}

impl ParsedArgs {
    fn new(
        use_gui: bool,
        gui_mode: String,
        path: String,
        custom_title: String,
        verbose: bool,
    ) -> Self {
        Self {
            use_gui,
            gui_mode,
            path,
            custom_title,
            verbose,
        }
    }

    /// Builds a struct of arguments
    fn build() -> ParsedArgs {
        let match_results = command!()
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('v')
                    .help("Print verbose information")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("gui")
                    .long("gui")
                    .short('g')
                    .help("Select subtitle from a dialog")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("custom_title")
                    .long("custom_title")
                    .short('c')
                    .help("Use a Custom title diofferent than the file name, (no hash)")
            )
            .arg(Arg::new("movie").required(true))
            .get_matches();

        // -- parse arguments
        let verbose = match_results.get_flag("verbose");

        let mut use_gui = false;
        let mut gui_mode = Default::default();

        if match_results.get_flag("gui") {
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

        let custom_title: String;
        if let Some(name) = match_results.get_one::<String>("custom_title") {
            custom_title = name.to_owned();
        } else {
            custom_title = "".to_owned();
        }

        let file: &String = match_results.get_one::<String>("movie").unwrap();
        //Returns struct of ParsedArgs
        ParsedArgs::new(use_gui, gui_mode, file.to_string(), custom_title, verbose)
    }
}

fn run(parsed_args: ParsedArgs, config: Config) -> Result<(), Box<dyn Error>> {
    //Gets movie properties
    let movie = Movie::build(&parsed_args.path, &parsed_args.custom_title)?;

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
