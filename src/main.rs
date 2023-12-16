use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use std::collections::HashMap;
use std::io::{self, Read};
use std::process::Command;
use serde::Deserialize;
use std::path::PathBuf;
use serde_json::Value;
use std::error::Error;
use getopts::Options;
use serde::Serialize;
use std::{
    env,
    fs::File,
    io::{Seek, SeekFrom},
    mem,
    path::Path,
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
    fn build() -> Result<Config, Box<dyn Error>> {
        let config = Config {
            key: "".to_owned(),
            user: "".to_owned(),
            password: "".to_owned(),
            language: "en".to_owned(),
            useragent: "opensubtitles downloader".to_owned(),
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

        let config = if let Ok(content) = std::fs::read(&config_path) {
            let config: Config = toml::from_str(&String::from_utf8(content)?)?;

            if config.user.is_empty()
                || config.key.is_empty()
                || config.password.is_empty()
                || config.language.is_empty()
            {
                return Err("Config file osd.conf fields cannot be empty.")?;
            }
            config
        } else {
            //if config file not found create a empty one
            let config = Config::build()?;
            config
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
    path: String,
}

impl ParsedArgs {
    /// Builds a struct of arguments
    fn build(args: &[String]) -> ParsedArgs {
        // -- parse arguments
        let mut opts = Options::new();
        opts.optflag("g", "gui", "Select subtitle from a list");
        opts.optflag("h", "help", "prints this help");

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("{}", f);
                print_help();
                std::process::exit(1);
            }
        };

        //prints help and exits
        if matches.opt_present("h") {
            print_help();
            std::process::exit(1);
        };

        let mut opt_gui = false;
        if matches.opt_present("g") {
            opt_gui = true;
        }

        let free_args = matches.free.len();
        //Only accepts one argument, the movie filename
        if free_args != 1 {
            print_help();
            std::process::exit(1);
        }

        //Returns struct of ParsedArgs
        return ParsedArgs {
            use_gui: opt_gui,
            path: matches.free.first().unwrap().to_string(),
        };
    }
}

///Movie properties
struct Movie {
    path: String,
    title: String,
    hash: String,
}

impl Movie {
    ///Movie properties builder
    fn build(path: &str) -> Result<Movie, Box<dyn Error>> {
        let path = Path::new(path);

        //Checking file exists
        if !path.exists() {
            print_help();
            return Err("File not found.")?;
        }

        let path_movie;
        //concatenates the movie path
        if Path::new(path).has_root() {
            path_movie = Path::new(path).to_path_buf();
        } else {
            let curren_dir = env::current_dir().unwrap();
            path_movie = Path::new(&curren_dir).join(path);
        }

        if !path_movie.is_file() {
            print_help();
            return Err("")?;
        }

        //Getting movie filename from path
        let movie_title = path_movie
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        //Path to String
        let path_movie = path_movie.to_str().unwrap_or_default().to_owned();

        //hash used by opensubtitles
        let hash = Movie::create_hash(&path_movie)?;

        //Returns movie struct
        Ok(Movie {
            path: path_movie,
            title: movie_title,
            hash,
        })
    }

    ///computes hash from a file
    fn create_hash(path: &str) -> Result<String, std::io::Error> {
        const HASH_BLK_SIZE: u64 = 65536;
        let file = File::open(path)?;
        let fsize = file.metadata().unwrap().len();

        let mut buf = [0u8; 8];
        let mut word: u64;

        let mut hash_val: u64 = fsize; // seed hash with file size

        if fsize < HASH_BLK_SIZE {
            return Err(std::io::Error::new(
                io::ErrorKind::Other,
                "File size too small.",
            ));
        }
        let iterations = HASH_BLK_SIZE / 8;

        let mut reader = std::io::BufReader::with_capacity(HASH_BLK_SIZE as usize, file);

        for _ in 0..iterations {
            reader.read(&mut buf)?;
            unsafe {
                word = mem::transmute(buf);
            };
            hash_val = hash_val.wrapping_add(word);
        }

        reader.seek(SeekFrom::Start(fsize - HASH_BLK_SIZE))?;

        for _ in 0..iterations {
            reader.read(&mut buf)?;
            unsafe {
                word = mem::transmute(buf);
            };
            hash_val = hash_val.wrapping_add(word);
        }

        let hash_string = format!("{:01$x}", hash_val, 16);

        Ok(hash_string)
    }
}

/// prints help
fn print_help() {
    println!("usage: osd [-h] [--gui] movie_file");
}

///Obtains movie id from opensubtitles from hash or movie filename.
fn search_for_subtitle_id_key(
    query: &str,
    hash: &str,
    key: &str,
    language: &str,
    from_gui: bool,
    user_agent: &str,
) -> Result<String, Box<dyn Error>> {
    let params = [
        ("languages", language),
        ("query", query),
        ("moviehash", hash),
    ];

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Api-Key", HeaderValue::from_str(key)?);

    //makes a request
    const URL: &str = "https://api.opensubtitles.com/api/v1/subtitles";
    let client = reqwest::blocking::Client::new();
    let urlwp = reqwest::Url::parse_with_params(URL, params)?;
    let resp = client.get(urlwp).headers(headers).send()?.text()?;

    let json: Value = serde_json::from_str(&resp)?;

    //If no subtitles found, exit
    let total_count = json["total_count"].as_i64().unwrap_or(0);
    if total_count < 1 {
        return Err("No subtitles found.")?;
    }

    // Shows a selection movie list
    if from_gui == true {
        //Storing filename with key
        let mut filename_map: HashMap<String, i64> = HashMap::new();

        let mut v_titles: Vec<(String, String)> = Vec::new();

        let json_array = json["data"].as_array().unwrap();
        for n in json_array {
            let filename = n["attributes"]["files"][0]["file_name"]
                .as_str()
                .unwrap_or("")
                .to_string();

            filename_map.insert(
                filename.clone(),
                n["attributes"]["files"][0]["file_id"].as_i64().unwrap_or(0),
            );

            let s = if n["attributes"]["moviehash_match"] == true {
                "✅".to_string()
            } else {
                "".to_string()
            };
            v_titles.push((filename, s));
        }

        let mut zenity_process = Command::new("zenity");
        zenity_process.args([
            "--width=720",
            r#"--height=400"#,
            r#"--list"#,
            r#"--title=Choose a subtitle"#,
            "--column=Subtitle",
            r#"--column=Hash Match"#,
        ]);

        //Adds to zinnity the column values
        for n in v_titles {
            zenity_process.arg(n.0);
            zenity_process.arg(n.1);
        }
        let out = zenity_process.output()?;

        let status_code = out.status.code().unwrap_or(1);
        if status_code == 1 {
            return Err("Movie not selected.")?;
        } else {
            let movie_selected = std::str::from_utf8(&out.stdout)?.trim_end_matches('\n');
            let key = filename_map.get(movie_selected);
            Ok(key.unwrap().to_string())
        }
    } else {
        //Looks for a hash match
        for n in json.get("data").iter() {
            if n["attributes"]["moviehash_match"] == true {
                return Ok(n["attributes"]["files"][0]["file_id"].to_string());
            }
        }
        //If not the first id of the list
        Ok(json["data"][0]["attributes"]["files"][0]["file_id"].to_string())
    }
}

/// gets login token
fn login(
    key: &str,
    user: &str,
    password: &str,
    user_agent: &str,
) -> Result<String, Box<dyn Error>> {
    let mut payload = HashMap::new();
    payload.insert("username", user);
    payload.insert("password", password);

    let payload = serde_json::to_string(&payload)?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Api-Key", HeaderValue::from_str(key)?);

    //makes a request
    const URL: &str = "https://api.opensubtitles.com/api/v1/login";
    let url = reqwest::Url::parse(URL)?;
    let client = reqwest::blocking::Client::new();
    let resp = client.post(url).body(payload).headers(headers).send()?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(format!("Bad request: {}", resp.status()))?;
    }

    let resp = resp.text()?;
    let rej: Value = serde_json::from_str(&resp)?;
    Ok(rej["token"].to_string())
}

///Request a download url for a subtitle.
fn download_url(
    file_id: &str,
    token: &str,
    key: &str,
    user_agent: &str,
) -> Result<String, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Api-Key", HeaderValue::from_str(key)?);
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    let mut payload = HashMap::new();
    payload.insert("file_id", file_id);
    let payload = serde_json::to_string(&payload)?;

    //makes a request
    const URL: &str = "https://api.opensubtitles.com/api/v1/download";
    let urlwp = reqwest::Url::parse(URL)?;
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(urlwp)
        .body(payload)
        .headers(headers)
        .send()?
        .text()?;

    let rej: Value = serde_json::from_str(&resp)?;
    Ok(rej["link"].to_string())
}

fn download_save_file(sub_url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let mut sub_path = PathBuf::from(path);
    sub_path.set_extension("srt");

    //Remove start end quotes. Why?
    let p = &sub_url[1..sub_url.len() - 1];

    let url = reqwest::Url::parse(p)?;
    let mut resp = reqwest::blocking::get(url)?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(format!("Bad request: {}", resp.status()))?;
    }

    let mut file_path = File::create(sub_path)?;
    io::copy(&mut resp, &mut file_path)?;

    Ok(())
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
        &config.useragent,
    )?;

    let token = login(
        &config.key,
        &config.user,
        &config.password,
        &config.useragent,
    )?;
    // download suitable subtitle
    let sub_url = download_url(&file_id, &token, &config.key, &config.useragent)?;
    download_save_file(&sub_url, &movie.path)?;

    Ok(())
}

fn main() -> Result<(),Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    let config = Config::load_config()?;
    //parse arg to a convenient struct
    let parsed_args = ParsedArgs::build(&args);

    match run(parsed_args, config) {
        Ok(_) => eprintln!("Done"),
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
