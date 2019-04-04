/*
Small program that pipes directory content into a dynamic menu and then opens your choice in a video player.
Meant to be slightly faster than using a bash script.
*/

use std::process::{Command, Stdio};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;

struct DynamicMenu<'a> { name: &'a str, args: &'a [&'a str] }
struct VideoPlayer<'a> { name: &'a str, args: &'a [&'a str] }

/* /path/to/dir */
static DIR: &str = "/home/kim/Film";

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
        let episodes = read_dir().unwrap_or_else(|e| panic!("Error: {}", e));

        let stdin = dmenu.stdin.as_mut().unwrap_or_else(|| panic!("Error: failed to open stdin pipe for {}", DYNAMIC_MENU.name));
        for episode in episodes {
            stdin.write_all(&episode[..]).unwrap_or_else(|e| panic!("Error: {}", e));
        }
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

fn read_dir() -> std::io::Result<Vec<Vec<u8>>> {
    let mut episodes: Vec<_> = std::fs::read_dir(DIR)?
        .flat_map(|result| {
            result.map(|entry| {
                let mut entry: Vec<u8> = entry.file_name().as_bytes().to_vec();
                entry.push(10); // 10 = "\n"
                entry
            })
        }).collect();

    episodes.sort();
    Ok(episodes)
}
