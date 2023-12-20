use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::{env, mem};
use std::{fs::File, process::Command};

///Movie properties
pub struct Movie {
    pub path: String,
    pub title: String,
    pub hash: String,
}

impl Movie {
    ///Movie properties builder
    pub fn build(path: &str) -> Result<Movie, Box<dyn Error>> {
        let path = Path::new(path);

        //Checking file exists
        if !path.exists() {
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
            if path_movie.is_dir() {
                return Err("The path must point to a file.")?;
            }
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
    pub fn create_hash(path: &str) -> Result<String, std::io::Error> {
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
/// Obtains opensubtitles id key using zenity list
fn process_id_key_with_zenity(json_array: &Vec<Value>) -> Result<String, Box<dyn Error>> {
    let mut filename_map: HashMap<String, i64> = HashMap::new();
    let mut v_titles: Vec<(String, String)> = Vec::new();

    for n in json_array {
        let filename = n["attributes"]["files"][0]["file_name"]
            .as_str()
            .unwrap_or("")
            .to_string();

        filename_map.insert(
            filename.clone(),
            n["attributes"]["files"][0]["file_id"].as_i64().unwrap_or(0),
        );

        let moviehash = if n["attributes"]["moviehash_match"] == true {
            "✅".to_string()
        } else {
            "".to_string()
        };
        //saves filename and hash on a tuple vector
        v_titles.push((filename, moviehash));
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
    //0: movi selected, !=0: cancel button
    if status_code == 1 {
        return Err("Movie not selected.")?;
    } else {
        let movie_selected = std::str::from_utf8(&out.stdout)?.trim_end_matches('\n');
        let file_id = filename_map.get(movie_selected);
        //Returs file_id
        Ok(file_id.unwrap().to_string())
    }
}
///Obtains movie id from opensubtitles from hash or movie filename.
pub fn search_for_subtitle_id_key(
    query: &str,
    hash: &str,
    key: &str,
    language: &str,
    use_gui: bool,
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

    //to json
    let json: Value = serde_json::from_str(&resp)?;

    //If no subtitles found, exit
    let total_count = json["total_count"].as_i64().unwrap_or(0);
    if total_count < 1 {
        return Err("No subtitles found.")?;
    }

    // Shows a selection movie list
    if use_gui == true {
        let json_array = json["data"].as_array().unwrap();

        let file_id = process_id_key_with_zenity(json_array)?;

        Ok(file_id)
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
pub fn login(
    key: &str,
    user: &str,
    password: &str,
    user_agent: &str,
) -> Result<String, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Api-Key", HeaderValue::from_str(key)?);

    //makes a request
    const URL: &str = "https://api.opensubtitles.com/api/v1/login";
    let url = reqwest::Url::parse(URL)?;
    let client = reqwest::blocking::Client::new();

    let mut payload = HashMap::new();
    payload.insert("username", user);
    payload.insert("password", password);
    let payload = serde_json::to_string(&payload)?;

    let resp = client.post(url).body(payload).headers(headers).send()?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(format!("Bad request: {}, {}", resp.status(), resp.text()?))?;
    }

    let resp = resp.text()?;
    let rej: Value = serde_json::from_str(&resp)?;
    Ok(rej["token"].to_string())
}

///Request a download url for a subtitle.
pub fn download_url(
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

pub fn download_save_file(sub_url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let mut sub_path = PathBuf::from(path);
    sub_path.set_extension("srt");

    //Remove start end quotes. Don't know why.
    let p = &sub_url[1..sub_url.len() - 1];

    let url = reqwest::Url::parse(p)?;
    let mut resp = reqwest::blocking::get(url)?;

    if resp.status() != reqwest::StatusCode::OK {
        return Err(format!("Bad request: {}", resp.status()))?;
    }

    //Save the subtitle
    let mut file_path = File::create(sub_path)?;
    io::copy(&mut resp, &mut file_path)?;

    Ok(())
}
