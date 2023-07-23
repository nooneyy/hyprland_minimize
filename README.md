# hyprland_minimize

hyprland_minimize is a script/app for Rofi and Hyprland written in Rust that allows you to switch focus between windows and "unminimize" those from the special:minimized workspace.

***DISCLAIMER***: This is my first Rust project so the code won't be amazing. It isn't production-ready software, use at your own discretion!

## Explanation
Whenever the script is run by Rofi or without arguments, it outputs the **windows with a non-empty class** in this format:
```
# If class == title:
{monitor}: {class}, Workspace: {workspace_name}

# Else:
{monitor}: {class} >> {title}, Workspace: {workspace_name}
```

If you pass one of these outputs to the script as an argument, e.g.:
```bash
./hyprland_minimize "0: Spotify, Workspace: special:minimized"
```
It does the following things:
- If the selected window is in a special Hyprland workspace called *minimized*, it moves it to the workspace of your focused window. If your focused window before that was fullscreened, it unfullscreens it.
- If the selected window is in other workspaces, it switches focus to it. If your focused window before that and your selected window are in the same workspace and the focused window was fullscreened, it unfullscreens it.

## Installation

Download the latest release from [GitHub Releases](https://github.com/nooneyy/hyprland_minimize/releases) or build it yourself:

```bash
git clone https://github.com/nooneyy/hyprland_minimize
cd hyprland_minimize
cargo build --release
```
and then copy it to wherever you want to execute it from!
```bash
cd target/release/
cp hyprland_minimize ~/.config/
```

## Example Usage

> hyprland.conf
```
...
# Bind for moving the focused window to the special:minimized workspace ("minimizing" a window)
bind = $mainMod, H, movetoworkspace, special:minimized

# Bind for opening hyprland_minimize through rofi (where hyprland_minimize is located at ~/.config/hyprland_minimize)
bind = $mainMod, W, exec, rofi -show script -modes "script:~/.config/hyprland_minimize"
...
```
In this case, mod+H would move a focused window to the special:minimized workspace (effectively minimizing it) and mod+W would open the rofi menu that executes hyprland_minimize.

## Contributing

Pull requests are welcome. If you want something to be added, open an issue or fork this repo and experiment with it and try to do it yourself!

## License

[MIT](https://github.com/nooneyy/hyprland_minimize/blob/main/LICENSE)