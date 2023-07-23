use std::env;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
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
        let mut formatted: String = String::new();
        if !(self.class.is_empty()) {
            let (_, ws_name) = &self.workspace;
            if self.class == self.title {
                formatted = format!(
                    "{}: {}, Workspace: {}",
                    self.monitor,
                    self.class,
                    ws_name.trim_matches(|c| c == '(' || c == ')')
                );
            } else {
                formatted = format!(
                    "{}: {} >> {}, Workspace: {}",
                    self.monitor,
                    self.class,
                    self.title,
                    ws_name.trim_matches(|c| c == '(' || c == ')')
                );
            }
        }
        match formatted.as_str() {
            "" => None,
            _ => Some(formatted),
        }
    }
}

fn main() {
    let windows: Vec<Window> = parse_clients(&hyprctl_command(vec!["clients"]).unwrap_or_default());
    if env::args().len() == 2 {
        let arg: String = env::args()
            .collect::<Vec<String>>()
            .get(1)
            .unwrap_or(&String::new())
            .to_owned();
        if let Some(window) = get_window_from_arg(&windows, &arg) {
            let (_, ws_name) = window.workspace;
            if ws_name == "(special:minimized)" {
                unminimize(window.pid);
            } else {
                focus(window.pid);
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

fn hyprctl_command(args: Vec<&str>) -> Result<String, String> {
    let output: std::process::Output = Command::new("hyprctl")
        .args(&args)
        .output()
        .unwrap_or_else(|_| panic!("Failed to execute hyprctl with args {:#?}", &args));
    String::from_utf8(output.stdout).map_err(|e| e.to_string())
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

fn get_window_from_arg(windows: &Vec<Window>, arg: &String) -> Option<Window> {
    let mut window_result = Window::new();
    for window in windows {
        if let Some(x) = window.format() {
            if x == *arg {
                window_result = window.clone();
                break;
            }
        }
    }
    if Window::new() != window_result {
        Some(window_result)
    } else {
        None
    }
}

fn unminimize(pid: i32) {
    if let Some(window) =
        parse_clients(&hyprctl_command(vec!["activewindow"]).unwrap_or_default()).get(0)
    {
        if window.fullscreen > 0 {
            let (workspace, _) = window.workspace;
            if let Err(e) = hyprctl_command(vec![
                "dispatch",
                "fullscreen",
                &format!("pid:{}", window.pid),
            ]) {
                eprintln!("Error un-fullscreening focused window: {}", e);
            }
            if let Err(e) = hyprctl_command(vec![
                "dispatch",
                "movetoworkspace",
                &format!("{},pid:{pid}", workspace),
            ]) {
                eprintln!("Failed to move minimized window to workspace: {}", e);
            }
        } else {
            let (workspace, _) = window.workspace;
            if let Err(e) = hyprctl_command(vec![
                "dispatch",
                "movetoworkspace",
                &format!("{},pid:{pid}", workspace),
            ]) {
                eprintln!("Failed to move minimized window to workspace: {}", e);
            }
        }
    }
}

fn focus(pid: i32) {
    if let Some(window) =
        parse_clients(&hyprctl_command(vec!["activewindow"]).unwrap_or_default()).get(0)
    {
        if window.fullscreen > 0 {
            if let Err(e) = hyprctl_command(vec![
                "dispatch",
                "fullscreen",
                &format!("pid:{}", window.pid),
            ]) {
                eprintln!("Error un-fullscreening focused window: {}", e);
            }
            if let Err(e) = hyprctl_command(vec!["dispatch", "focuswindow", &format!("pid:{pid}")])
            {
                eprintln!("Failed to move focus window: {}", e);
            }
        } else if let Err(e) =
            hyprctl_command(vec!["dispatch", "focuswindow", &format!("pid:{pid}")])
        {
            eprintln!("Failed to move focus window: {}", e);
        }
    }
}
