use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::fs::File;
use std::fs;
use std::env;

use std::{process, collections::HashMap};
use regex::Regex;

use inquire::{InquireError, Select};
use indicatif::{ProgressBar, ProgressStyle};
use flate2::read::GzDecoder;
use tar::Archive;
use std::{path::PathBuf};

use neon::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ListItem {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub cookie: Option<String>,
    pub list: Option<Vec<ListItem>>,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            cookie: None,
            list: None,
        }
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("parser", parser)?;
    Ok(())
}

pub fn write_to_see(key: &str, value: impl Serialize) -> Result<(), Box<dyn Error>> {
    let home_dir = env::var("HOME")?;
    let see_file_path = format!("{}/.see", home_dir);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(see_file_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Data = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("deserialize error: {}", e);
            Default::default()
        }
    };

    match key {
        "cookie" => {
            let cookie: Option<String> = match serde_json::from_value(serde_json::to_value(value)?) {
                Ok(cookie) => cookie,
                Err(e) => {
                    eprintln!("deserialize error: {}", e);
                    None
                }
            };
            data.cookie = cookie;
        }
        "list" => {
            let list: Option<Vec<ListItem>> = match serde_json::from_value(serde_json::to_value(value)?) {
                Ok(list) => list,
                Err(e) => {
                    eprintln!("deserialize error: {}", e);
                    None
                }
            };
            if let Some(new_list) = list {
                if let Some(existing_list) = &mut data.list {
                    for new_item in new_list {
                        if let Some(existing_item) = existing_list.iter_mut().find(|item| item.name == new_item.name) {
                            existing_item.value = new_item.value.clone();
                        } else {
                            existing_list.push(new_item);
                        }
                    }
                } else {
                    data.list = Some(new_list);
                }
            }
        }
        _ => return Err("Invalid key".into()),
    }

    file.set_len(0)?;
    file.seek(std::io::SeekFrom::Start(0))?;
    serde_json::to_writer(file, &data).unwrap();

    Ok(())
}

pub fn read_from_see() -> Result<Data, Box<dyn Error>> {
    let home_dir = env::var("HOME")?;
    let see_file_path = format!("{}/.see", home_dir);
    let mut file = match File::open(see_file_path) {
        Ok(file) => file,
        Err(_) => return Ok(serde_json::from_str("{}")?),
    };
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let data: Data = serde_json::from_str(&contents)?;

    Ok(data)
}

fn is_dir_not_empty(path: &str) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            // 如果有任何一个文件，就说明目录非空
            if entry.is_ok() {
                return true;
            }
        }
    }
    false
}

fn is_dir_exist(path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.exists() && path.is_dir()
}

pub fn has_content_in_dir(path: &str) -> bool {
    is_dir_exist(path) && is_dir_not_empty(path)
}


pub fn parser(mut cx: FunctionContext) -> JsResult<JsString> {
    let mut data = read_from_see().unwrap();

    let options: Vec<&str> = match data.list {
        Some(ref list) => list.iter().map(|elem| elem.name.as_str()).collect(),
        None => Vec::new(),
    };

    let args: Vec<String> = env::args().skip(2).collect();
    let config = ParseConfig::new(&args);

    if ParseConfig::is_inner_cmd(&config.cmd) {
        match config.cmd.as_str() {
            "select" => {
                let ans: Result<&str, InquireError> = Select::new("Choose a project as a template:", options).prompt();

                let not_fonud = "not found".to_owned();
                let value = match ans {
                    Ok(ans) => {
                        data.list.as_ref().map_or(None, |l| {
                            l.iter().find_map(|i| {
                                if i.name == ans {
                                    Some(&i.value)
                                } else {
                                    None
                                }
                            })
                        }).unwrap_or(&not_fonud)
                    },
                    Err(_) => {
                        println!("Has no local repository map in $HOME/.see.");
                        process::exit(1)
                    }
                };

                let url_info = ParseConfig::parse_site(&value).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                    process::exit(1)
                });
                match try_download(&url_info, &config.dest, &data.cookie) {
                    Ok(_result) => {
                        // 操作成功，可以继续处理结果
                        println!("Download succeeded");
                    }
                    Err(error) => {
                        // 操作失败，处理错误情况
                        eprintln!("Download failed: {}", error);
                    }
                }
            },
            "set-token" => {
                match write_to_see("cookie", config.query) {
                    Ok(()) => {
                        // 操作成功
                        println!("Write to .see succeeded.");
                    }
                    Err(error) => {
                        // 操作失败，处理错误情况
                        eprintln!("Write to .see failed: {}.", error);
                    }
                }
            },
            "pull" => {
                let url_info = ParseConfig::parse_site(&config.query).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                    process::exit(1)
                });
                match try_download(&url_info, &config.dest, &data.cookie) {
                    Ok(result) => {
                        // 操作成功，可以继续处理结果
                        println!("Download succeeded: {:?}", result);
                    }
                    Err(error) => {
                        // 操作失败，处理错误情况
                        eprintln!("Download failed: {}", error);
                    }
                }
                let new_item: ListItem = ListItem { name: url_info.name, value: url_info.url };

                if data.list.is_none() {
                    data.list = Some(vec![]);
                }
                data.list.as_mut().unwrap().push(new_item);
                match write_to_see("list", data.list) {
                    Ok(()) => {
                        // 操作成功
                        println!("Write to .see succeeded.");
                    }
                    Err(error) => {
                        // 操作失败，处理错误情况
                        eprintln!("Write to .see failed: {}.", error);
                    }
                }
            },
            _ => panic!("Invalid key"),
        };
    } else {
        println!("no valid query {}", &config.cmd)
    }
    Ok(cx.string("hello node"))
}


struct ParseConfig {
    cmd: String,
    query: String,
    dest: PathBuf
}

#[derive(Debug)]
struct UrlInfo {
    site: String,
    _user: String,
    name: String,
    _refer: String,
    url: String,
    _ssh: String,
    _subdir: String,
    _mode: String
}

impl ParseConfig {
    fn new(args: &[String]) -> ParseConfig {
        let mut _cmd: String = String::from("select");
        let mut query: String = String::from("");
        let mut dest: PathBuf = PathBuf::from(".");

        match args.len() {
            0 => { /* 使用默认值 */ }
            1 => {
                if args[0] != "select" {
                    query = args[0].clone();
                    _cmd = String::from("pull");
                }
            }
            2 => {
                if !["select", "pull", "set-token"].contains(&args[0].as_str()) {
                    _cmd = String::from("pull");
                    query = args[0].clone();
                    dest = PathBuf::from(args[1].clone());
                } else {
                    _cmd = args[0].clone();
                    query = args[1].clone();
                    if args[0] == "select" {
                        dest = PathBuf::from(args[1].clone());
                    }
                }
            }
            _ => {
                _cmd = args[0].clone();
                query = args[1].clone();
                dest = PathBuf::from(args[2].clone());
            }
        };
        ParseConfig { cmd: _cmd, query, dest }
    }

    fn parse_site(src: &str) -> Result<UrlInfo, &'static str> {
        const VALID_SITES: [&str; 2] = ["github", "gitlab"];

        let re = Regex::new("^(?:(?:https://)?([^:/]+.[^:/]+)/|git@([^:/]+)[:/]|([^/]+):)?(?P<user>[^/s]+)/(?P<name>.+)(?P<subdir>:(?P<refer>(?:/[^/s#]+)+))?(?:/)?(?:#(.+))?")
        .unwrap();
        if re.is_match(src) {

            let caps: regex::Captures<'_> = re.captures(src).unwrap();
            let matched_site = caps.get(1).or(caps.get(2)).or(caps.get(3));
            let site: String;
            if let true = matched_site.is_some() {
                let reg = Regex::new(".(com|org)$").unwrap();
                site = reg.replace_all(matched_site.unwrap().as_str(), "").to_string();
            } else {
                site = String::from("github");
            }

            let mut res = false;
            let mut mode = String::from("git");
            for (_, v) in VALID_SITES.iter().enumerate() {
                if site.contains(v) {
                    res = true;
                    mode = String::from("tar");
                }
            }
            if !res && !site.starts_with("gitlab") {
                return Err("cli supports GitHub, GitLab");
            }

            let domain = site.clone() + ".com";
            
            let user = caps.name("user").unwrap().as_str().to_string();
            let name = caps.name("name").unwrap().as_str().replace(".git", "").to_string();
            let subdir;
            subdir = match caps.name("subdir") {
                None => "xxx",
                Some(i) => i.as_str(),
            }.to_string();
            let refer;
            refer = match caps.name("refer") {
                None => "HEAD",
                Some(i) => i.as_str(),
            }.to_string();
            let url = format!("https://{domain}/{user}/{name}");
            let ssh = format!("git@{domain}:{user}/{name}");

            Ok(UrlInfo {site, _user: user, name, _refer: refer, url, _ssh: ssh, _subdir: subdir, _mode: mode})
        } else {
            return Err("Not enough arguments.");
        }
    }
    fn is_inner_cmd(cmd: &str) -> bool {
        let mut cmds = HashMap::new();

        cmds.insert(String::from("select"), true);
        cmds.insert(String::from("set-token"), true);
        cmds.insert(String::from("pull"), true);

        cmds.get(cmd).copied().unwrap_or(false)
    }
}

fn download(url_info: &UrlInfo, dest: &PathBuf, token: &Option<String>) -> Result<(), Box<dyn Error>> {
    const HASH: &str = "HEAD";
    let url = if url_info.site.as_str().contains("github") {
        format!("{}/archive/{}.tar.gz", url_info.url, HASH)
    } else if url_info.site.as_str().contains("gitlab") {
        format!("{}/-/archive/{}/{}.tar.gz?private_token={}", url_info.url, HASH, url_info.name, token.clone().unwrap_or_default())
    } else {
        panic!("Site provider not supported.")
    };

    let client: reqwest::Client = reqwest::Client::new();

    let request = client.get(&url).send().unwrap();
    match request.status() {
        reqwest::StatusCode::OK => {
            if request.url().to_string().contains("sign_in") {
                println!("UNAUTHORIZED: Please set private_token, can use see set-token <your access-token>");
                process::exit(1);
            }
            ()
        },
        reqwest::StatusCode::UNAUTHORIZED => {
            panic!("Could not find repository.");
        }
        s => {
            panic!("Received response status: {}", s);
        }
    };


    let total_size = request.content_length();

    let pb = match total_size {
        Some(x) => {
            let p = ProgressBar::new(x);
            p.set_style(ProgressStyle::default_bar()
                     .template("> {wide_msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                     .progress_chars("#>-"));
            p
        }
        None => {
            let p = ProgressBar::new_spinner();
            p
        }
    };

    println!("Downloading {} to {}", url_info.url, dest.display());

    let tar = GzDecoder::new(pb.wrap_read(request));
    let mut archive = Archive::new(tar);
    archive
        .entries().unwrap()
        .filter_map(|e| e.ok())
        .map(|mut entry| -> Result<PathBuf, Box<dyn Error>> {
            let path = entry.path()?;
            let path = path
                .strip_prefix(path.components().next().unwrap())?
                .to_owned();
            entry.unpack(dest.join(&path))?;
            Ok(path)
        })
        .filter_map(|e| e.ok())
        .for_each(|x| pb.set_message(&format!("{}", x.display())));

    // archive.unpack(dest).unwrap();
    pb.finish_with_message("Done...");
    Ok(())
}

fn try_download(url_info: &UrlInfo, dest: &PathBuf, token: &Option<String>) -> Result<(), Box<dyn Error>> {
    let has_no_dir = !has_content_in_dir(dest.to_str().unwrap());
    if has_no_dir {
        return download(url_info, dest, token);
    } else {
        println!("The current directory({}) contains files.", dest.display());
        let ans: Result<&str, InquireError> = Select::new("What do you expect to do?", vec!["overwrite", "quit"]).prompt();
        match ans.unwrap() {
            "overwrite" => {
                return download(url_info, dest, token);
            },
            "quit" => {
                process::exit(1);
            },
            _ => panic!()
        }
    }
}