use std::env;
#[cfg(target_os = "macos")]
use std::fs;
use std::io::{self, Write};
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::process::Command;
use std::thread;
use std::time::Duration;

const ENV_NAME: &str = "HF_ENDPOINT";
const MAINLAND_ENDPOINT: &str = "https://hf-mirror.com";
const OVERSEAS_ENDPOINT: &str = "http://huggingface.co";

const BILIBILI_CHANNEL: &str = "https://space.bilibili.com/693604061";
const PANPANXIA_SITE: &str = "https://www.panpanxia.com";
const VOICEBOX_REPO: &str = "https://ccn4c6wxmlj9.feishu.cn/wiki/OiOHwycefi6uQJkHqHpcz2v4nOe";

fn main() {
    println!("================ Tech指南 =======================");
    println!("我的B站：\n\t{BILIBILI_CHANNEL}");
    println!();
    println!("找到你想找的资源：\n\t{PANPANXIA_SITE}");
    println!();
    println!("更多声音克隆资料 👇:\n\t{VOICEBOX_REPO}");
    println!("================================================");
    println!();

    show_current_config();

    match prompt_region() {
        Ok(region) => {
            let endpoint = region.endpoint();
            match persist_hf_endpoint(endpoint) {
                Ok(()) => {
                    println!();
                    // println!("已将 {ENV_NAME} 永久设置为：{endpoint}");
                    println!("当前终端可能需要重启后才会读取到最新配置。");
                    println!();
                    println!(">>>>>>> 程序将在 5 秒后自动退出... <<<<<<<<");
                    thread::sleep(Duration::from_secs(5));
                }
                Err(error) => {
                    eprintln!();
                    eprintln!("设置失败：{error}");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("读取输入失败：{error}");
            std::process::exit(1);
        }
    }
}

enum Region {
    Mainland,
    Overseas,
}

impl Region {
    fn endpoint(&self) -> &'static str {
        match self {
            Self::Mainland => MAINLAND_ENDPOINT,
            Self::Overseas => OVERSEAS_ENDPOINT,
        }
    }
}

fn show_current_config() {
    let current_endpoint = env::var(ENV_NAME).unwrap_or_default();
    let current_region = match current_endpoint.as_str() {
        "" | OVERSEAS_ENDPOINT => "海 外 地 区",
        MAINLAND_ENDPOINT => "中 国 大 陆 地 区",
        _ => "未 知 地 区",
    };

    // println!("当前 {ENV_NAME}：{}", display_endpoint(&current_endpoint));
    if current_endpoint.is_empty() {
        println!("当前配置为空");
    } else {
        println!("当前配置不为空");
    }
    println!("当前配置为：{current_region}");
    println!();
}

fn display_endpoint(endpoint: &str) -> &str {
    if endpoint.is_empty() {
        "未设置"
    } else {
        endpoint
    }
}

fn prompt_region() -> io::Result<Region> {
    loop {
        println!("请选择你当前所处地区：");
        println!("1. 中 国 大 陆");
        println!("2. 海 外 地 区");
        print!("请输入 1 或 2 后按回车：");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => return Ok(Region::Mainland),
            "2" => return Ok(Region::Overseas),
            _ => {
                println!();
                println!("输入无效，请重新选择。");
                println!();
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn persist_hf_endpoint(endpoint: &str) -> io::Result<()> {
    let status = Command::new("setx").args([ENV_NAME, endpoint]).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("setx 执行失败，退出码：{status}"),
        ))
    }
}

#[cfg(target_os = "macos")]
fn persist_hf_endpoint(endpoint: &str) -> io::Result<()> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "未找到 HOME 目录"))?;

    let shell_profile = preferred_shell_profile(&home);
    upsert_export_line(&shell_profile, endpoint)
}

#[cfg(target_os = "macos")]
fn preferred_shell_profile(home: &std::path::Path) -> PathBuf {
    let shell = env::var("SHELL").unwrap_or_default();

    if shell.ends_with("zsh") {
        home.join(".zshrc")
    } else if shell.ends_with("bash") {
        home.join(".bash_profile")
    } else {
        home.join(".zshrc")
    }
}

#[cfg(target_os = "macos")]
fn upsert_export_line(profile_path: &std::path::Path, endpoint: &str) -> io::Result<()> {
    let export_line = format!("export {ENV_NAME}=\"{endpoint}\"");
    let existing = match fs::read_to_string(profile_path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };

    let mut found = false;
    let mut lines = Vec::new();

    for line in existing.lines() {
        if line
            .trim_start()
            .starts_with(&format!("export {ENV_NAME}="))
        {
            lines.push(export_line.clone());
            found = true;
        } else {
            lines.push(line.to_string());
        }
    }

    if !found {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(String::from("# Hugging Face endpoint"));
        lines.push(export_line);
    }

    let mut next_content = lines.join("\n");
    next_content.push('\n');
    fs::write(profile_path, next_content)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn persist_hf_endpoint(_endpoint: &str) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "当前程序仅支持 Windows 和 macOS",
    ))
}
