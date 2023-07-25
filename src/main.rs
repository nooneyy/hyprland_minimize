use std::env;
use std::process::Command;

#[derive(Clone)]
struct Window {
    monitor: i32,
    class: String,
    title: String,
    workspace: (i32, String),
    pid: i32,
    fullscreen: u8,
}

impl Window {
    fn new() -> Window {
        Window {
            monitor: 0,
            class: "".to_owned(),
            title: "".to_owned(),
            workspace: (0, "".to_owned()),
            pid: 0,
            fullscreen: 0,
        }
    }
    fn format(&self) -> Option<String> {
        if !self.class.is_empty() {
            let (_, ws_name) = &self.workspace;
            Some(format!(
                "{}: {}{}, Workspace: {}",
                self.monitor,
                self.class,
                if self.class != self.title {
                    format!(" >> {}", self.title)
                } else {
                    "".to_string()
                },
                ws_name.trim_matches(|c| c == '(' || c == ')')
            ))
        } else {
            None
        }
    }
}

fn main() {
    let windows: Vec<Window> = parse_clients(&hyprctl_command(&["clients"]).unwrap_or_default());
    if env::args().len() == 2 {
        let arg: String = env::args()
            .collect::<Vec<String>>()
            .get(1)
            .unwrap_or(&String::new())
            .to_owned();
        if let Some(window) = get_window_from_arg(&windows, &arg) {
            let (_, ws_name) = window.workspace;
            if ws_name == "(special:minimized)" {
                if let Err(e) = unminimize(window.pid) {
                    panic!("Failed to unminimize window: {}", e);
                }
            } else if let Err(e) = focus(&windows, window.pid, &ws_name) {
                panic!("Failed to focus window: {}", e);
            }
        }
    }
    if env::args().len() == 1 {
        println!("\0prompt\x1fWindows");
        for window in windows {
            if let Some(x) = window.format() {
                println!("{x}")
            }
        }
    }
}

fn hyprctl_command(args: &[&str]) -> Result<String, String> {
    String::from_utf8(
        Command::new("hyprctl")
            .args(args)
            .output()
            .unwrap_or_else(|_| panic!("Failed to execute hyprctl with arg {}", &args.join(" ")))
            .stdout,
    )
    .map_err(|e| e.to_string())
}

fn parse_clients(clients: &str) -> Vec<Window> {
    let mut parsed_clients: Vec<Window> = vec![];
    let mut split_clients: Vec<&str> = clients.split("\n\n").collect();
    split_clients.pop();
    for element in split_clients {
        let mut win = Window::new();
        let lines: Vec<&str> = element
            .lines()
            .filter(|ele: &&str| !(*ele).starts_with("Window "))
            .map(|line: &str| line.trim())
            .collect();
        for line in lines {
            if let Some((x, y)) = line.split_once(": ") {
                match x {
                    "monitor" => win.monitor = y.parse().unwrap_or_default(),
                    "class" => win.class = y.to_owned(),
                    "title" => win.title = y.to_owned(),
                    "workspace" => {
                        win.workspace = {
                            let (ws_num, ws_name) = y.split_once(' ').unwrap_or_default();
                            (ws_num.parse().unwrap_or_default(), ws_name.to_owned())
                        }
                    }
                    "pid" => win.pid = y.parse().unwrap_or_default(),
                    "fullscreen" => win.fullscreen = y.parse().unwrap_or_default(),
                    _ => (),
                }
            }
        }
        parsed_clients.push(win);
    }
    parsed_clients
}

fn get_window_from_arg(windows: &Vec<Window>, arg: &str) -> Option<Window> {
    for window in windows {
        if let Some(formatted) = window.format() {
            if formatted == arg {
                return Some(window.clone());
            }
        }
    }
    None
}

fn unminimize(pid: i32) -> Result<(), String> {
    if let Some(window) =
        parse_clients(&hyprctl_command(&["activewindow"]).unwrap_or_default()).get(0)
    {
        let (ws_int, _) = &window.workspace;
        if window.fullscreen > 0 {
            hyprctl_command(&["dispatch", "fullscreen"])?;
        }
        match hyprctl_command(&[
            "dispatch",
            "movetoworkspace",
            &format!("{},pid:{}", ws_int, pid),
        ]) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    } else if let Some(ws_name) =
        parse_activeworkspace(&hyprctl_command(&["activeworkspace"]).unwrap_or_default())
    {
        match hyprctl_command(&[
            "dispatch",
            "movetoworkspace",
            &format!("{},pid:{}", ws_name, pid),
        ]) {
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    } else {
        Err("Failed to get active workspace or active window".to_owned())
    }
}

fn focus(windows: &Vec<Window>, pid: i32, workspace: &str) -> Result<(), String> {
    for window in windows {
        let (_, ws_name) = &window.workspace;
        if window.fullscreen > 0 && workspace == *ws_name {
            println!("{}", &format!("pid:{}", window.pid));
            hyprctl_command(&["dispatch", "focuswindow", &format!("pid:{}", window.pid)])?;
            hyprctl_command(&["dispatch", "fullscreen"])?;
            break;
        }
    }
    match hyprctl_command(&["dispatch", "focuswindow", &format!("pid:{}", pid)]) {
        Err(e) => Err(e),
        Ok(_) => Ok(()),
    }
}

fn parse_activeworkspace(activeworkspace: &str) -> Option<&str> {
    activeworkspace
        .lines()
        .next()
        .and_then(|x| x.split_once(" on "))
        .and_then(|(y, _)| y.strip_prefix("workspace ID "))
        .and_then(|z| z.split_once(' ').map(|(_, a)| a))
        .map(|r| r.trim_matches(|c| c == '(' || c == ')'))
}
