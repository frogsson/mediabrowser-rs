/*
Small program that pipes directory content into a dynamic menu and then opens your choice in a video player.
Meant to be slightly faster than using a bash script.
*/

use std::process::{Command, Stdio};
use std::io::Write;

struct DynamicMenu<'a> { name: &'a str, args: &'a [&'a str] }
struct VideoPlayer<'a> { name: &'a str, args: &'a [&'a str] }

/* /path/to/dir */
static DIR: &str = "/home/kim/Shows";

static DYNAMIC_MENU: DynamicMenu = DynamicMenu {
    /* dmenu / rofi / fzf / fzy */

    name: "dmenu",
    args: &["-l", "14", "-i", "-p", "select"],

    // name: "fzf",
    // args: &[],

    // name: "rofi",
    // args: &["-dmenu", "-p", "select"],
};

static VIDEO_PLAYER: VideoPlayer = VideoPlayer {
    /* mpv / vlc */

    name: "mpv",
    args: &[],
};

fn main() {
    let mut dmenu = Command::new(DYNAMIC_MENU.name)
        .args(DYNAMIC_MENU.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to launch {}", DYNAMIC_MENU.name));

    {
        let stdin = dmenu.stdin.as_mut().expect("Failed to open stream");

        let episodes: String = read_dir()
            .unwrap_or_else(|e| panic!("{}", e))
            .iter()
            .flat_map(|s| s.chars())
            .collect();

        stdin.write_all(&episodes.as_bytes()).unwrap();
    }

    let dmenu_output = dmenu
        .wait_with_output()
        .expect("Failed to read stdout");

    match String::from_utf8(dmenu_output.stdout) {
        Ok(s) => {
            if !s.is_empty() {
                let ep = format!("{}/{}", DIR, s.trim_end_matches('\n'));
                Command::new(VIDEO_PLAYER.name)
                    .args(VIDEO_PLAYER.args)
                    .arg(ep)
                    .spawn()
                    .unwrap_or_else(|_| panic!("Failed to launch {}", VIDEO_PLAYER.name));
            }
        },
        Err(e) => eprintln!("{}", e),
    };
}

fn read_dir() -> std::io::Result<Vec<String>> {
    let mut episodes: Vec<String> = std::fs::read_dir(DIR)?
        .flat_map(|entry| entry.map(|e| e.file_name()))
        .map(|entry| {
            let mut entry = entry.to_str().map(String::from).unwrap();
            entry.push_str("\n");
            entry
        }).collect();

    episodes.sort();
    Ok(episodes)
}
