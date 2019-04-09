/*
Small program that pipes directory content into a dynamic menu and then opens your choice in a video player.
*/

use std::process::{Command, Stdio};
use std::io::Write;
use std::collections::{HashSet, HashMap, LinkedList};
use std::path::PathBuf;

struct DynamicMenu<'a> { name: &'a str, args: &'a [&'a str] }
struct VideoPlayer<'a> { name: &'a str, args: &'a [&'a str] }

// searches directories recursively
static DIRS: &[&str] = &[
    // /path/to/dir
    "/home/kim/Video",
];

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
    let mut map: HashMap<String, String> = HashMap::new();
    let mut dmenu = Command::new(DYNAMIC_MENU.name)
        .args(DYNAMIC_MENU.args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to launch {}", DYNAMIC_MENU.name));

    {
        let episodes = read_dir()
            .unwrap_or_else(|e| panic!("Error: {}", e));

        let stdin = dmenu.stdin.as_mut()
            .unwrap_or_else(|| panic!("Error: failed to open stdin pipe for {}", DYNAMIC_MENU.name));

        for episode in episodes.iter() {
            let e: Vec<&str> = episode.rsplitn(2, '/').collect();
            let file_name = e.get(0)
                .expect("Error: no file_name found");
            map.insert(file_name.to_string(), episode.to_string());

            stdin.write_all(file_name.as_bytes()).unwrap_or_else(|e| panic!("Error: {}", e));
        }
    }

    let dmenu_output = dmenu.wait_with_output()
        .expect("Failed to read stdout");

    match String::from_utf8(dmenu_output.stdout) {
        Ok(s) => {
            if !s.is_empty() {
                let ep = map[&s].trim_end_matches('\n');
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
    let mut episodes = Vec::with_capacity(500);
    let mut dir_stack: LinkedList<PathBuf> = LinkedList::new();
    let special_chars: Vec<_> = " !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".chars().collect();
    let filter: HashSet<&'static str> = [
        "mkv", "avi", "mp4", "webm", "flv",
        "vob", "ogv", "ogg", "drc", "f4b",
        "mng", "MTS", "M2TS", "mov", "qt",
        "wmv", "yuv", "rm", "rmvb", "asf",
        "amv", "m4p", "m4v", "mpg", "mp2",
        "mpeg", "mpe", "mpv", "mpg", "mpe",
        "m2v", "m4v", "svi", "3gp", "3g2",
        "mxf", "roq", "nsv", "f4v", "f4p",
        "f4a"].iter().cloned().collect();

    for dir in DIRS {
        let d = PathBuf::from(dir);

        if !d.is_dir() {
            panic!("Error: `{}` is not a Directory.", dir)
        }

        dir_stack.push_front(d);
    }

    while !dir_stack.is_empty() {
        let dir = dir_stack.pop_front()
            .expect("Error: no entry in dir_stack");

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?.path();

            if entry.is_dir() {
                dir_stack.push_front(entry)
            } else {
                let mut full_path = entry.to_str()
                    .map(ToString::to_string)
                    .unwrap();

                let file_name = full_path.rsplitn(2, '/')
                    .collect::<Vec<_>>()[0];

                let file_stem = file_name.splitn(2, '.')
                    .collect::<Vec<_>>()[0]
                    .to_lowercase();

                if file_stem == "sample" {
                    continue
                }

                let mut file_name_sort = file_name.to_lowercase();

                for ch in &special_chars {
                    file_name_sort.retain(|c| c != *ch)
                }

                if let Some(ext) = entry.extension() {
                    if filter.contains(ext.to_str().unwrap()) {
                        full_path.push('\n');
                        episodes.push((full_path, file_name_sort));
                    }
                }
            }
        }
    }

    episodes.sort_by(|a, b| a.1.cmp(&b.1));

    let episodes: Vec<_> = episodes.iter()
        .map(|t| t.0.to_string())
        .collect();

    Ok(episodes)
}
