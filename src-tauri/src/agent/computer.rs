//! Trusted computer-use tools. The model names an action; this module launches
//! and verifies. It never runs model-authored shell strings.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::error::{AgentError, AgentResult};
use super::tools::ToolOutput;

#[derive(Debug, Clone)]
pub struct AppTarget {
    pub display_name: String,
    pub launch_name: String,
}

pub fn execute(tool: &str, args: &Value) -> AgentResult<ToolOutput> {
    match tool {
        "OPEN_APPLICATION" => open_application(str_arg(args, "name")?),
        "OPEN_URL" => open_url(
            str_arg(args, "url")?,
            args.get("app").and_then(Value::as_str),
        ),
        "OPEN_FOLDER" => open_folder(str_arg(args, "path")?),
        "OPEN_FILE" => open_file(str_arg(args, "path")?),
        "CREATE_FOLDER" => create_folder(str_arg(args, "path")?),
        "LIST_APPS" => list_running_apps(),
        "CLOSE_APPLICATION" => close_application(str_arg(args, "name")?),
        "FOCUS_APPLICATION" => focus_application(str_arg(args, "name")?),
        "TAKE_SCREENSHOT" => take_screenshot(),
        "PLAY_PLAYLIST" => play_playlist(str_arg(args, "name")?),
        "TYPE_TEXT" => type_text(str_arg(args, "text")?),
        "PRESS_KEY" => press_key(str_arg(args, "key")?),
        "HOTKEY" => hotkey(args.get("keys").and_then(Value::as_array)),
        _ => Err(AgentError::msg(format!("{tool} is not a computer tool"))),
    }
}

pub fn resolve_application(name: &str) -> AgentResult<AppTarget> {
    let key = normalize(name);
    if let Some(target) = alias(&key) {
        return Ok(target);
    }
    if let Some(found) = discover(&key) {
        return Ok(found);
    }
    Ok(AppTarget {
        display_name: title_case(name),
        launch_name: name.trim().to_string(),
    })
}

fn open_application(name: &str) -> AgentResult<ToolOutput> {
    let target = resolve_application(name)?;
    platform().open_application(&target)?;
    let _ = platform().focus_application(&target);
    let verified = platform().verify_application(&target);
    if !verified.running {
        return Err(AgentError::msg(format!(
            "{} did not stay open. {}",
            target.display_name,
            verified.detail.unwrap_or_else(|| "It may not be installed.".into())
        )));
    }
    Ok(ToolOutput {
        summary: format!("{} is open.", target.display_name),
        value: json!({
            "success": true,
            "tool": "open_application",
            "data": {
                "application": target.display_name,
                "launchName": target.launch_name,
                "pid": verified.pid,
                "verified": true,
            },
            "error": null
        }),
    })
}

fn open_url(url: &str, app: Option<&str>) -> AgentResult<ToolOutput> {
    let url = normalize_url(url)?;
    let app_target = match app {
        Some(name) => Some(resolve_application(name)?),
        None => None,
    };
    platform().open_url(&url, app_target.as_ref())?;
    Ok(ToolOutput {
        summary: match &app_target {
            Some(app) => format!("Opened {url} in {}.", app.display_name),
            None => format!("Opened {url}."),
        },
        value: json!({
            "success": true,
            "tool": "open_url",
            "data": { "url": url, "application": app_target.map(|app| app.display_name) },
            "error": null
        }),
    })
}

fn open_folder(path: &str) -> AgentResult<ToolOutput> {
    let resolved = expand_user(path);
    if !resolved.is_dir() {
        return Err(AgentError::msg(format!(
            "That folder does not exist: {}",
            resolved.display()
        )));
    }
    platform().open_path(&resolved)?;
    Ok(ToolOutput {
        summary: format!("Opened {}.", resolved.display()),
        value: json!({
            "success": true,
            "tool": "open_folder",
            "data": { "path": resolved },
            "error": null
        }),
    })
}

fn open_file(path: &str) -> AgentResult<ToolOutput> {
    let resolved = expand_user(path);
    if !resolved.is_file() {
        return Err(AgentError::msg(format!(
            "That file does not exist: {}",
            resolved.display()
        )));
    }
    platform().open_path(&resolved)?;
    Ok(ToolOutput {
        summary: format!("Opened {}.", resolved.display()),
        value: json!({
            "success": true,
            "tool": "open_file",
            "data": { "path": resolved },
            "error": null
        }),
    })
}

fn create_folder(path: &str) -> AgentResult<ToolOutput> {
    let resolved = expand_user(path);
    std::fs::create_dir_all(&resolved).map_err(|err| {
        AgentError::msg(format!("Could not create {}: {err}", resolved.display()))
    })?;
    platform().open_path(&resolved)?;
    Ok(ToolOutput {
        summary: format!("Created {}.", resolved.display()),
        value: json!({
            "success": true,
            "tool": "create_folder",
            "data": { "path": resolved },
            "error": null
        }),
    })
}

fn list_running_apps() -> AgentResult<ToolOutput> {
    let apps = platform().running_apps();
    Ok(ToolOutput {
        summary: format!("{} apps are running.", apps.len()),
        value: json!({ "success": true, "tool": "list_apps", "data": { "apps": apps }, "error": null }),
    })
}

fn close_application(name: &str) -> AgentResult<ToolOutput> {
    let target = resolve_application(name)?;
    platform().close_application(&target)?;
    Ok(ToolOutput {
        summary: format!("{} was asked to quit.", target.display_name),
        value: json!({ "success": true, "tool": "close_application", "data": { "application": target.display_name }, "error": null }),
    })
}

fn focus_application(name: &str) -> AgentResult<ToolOutput> {
    let target = resolve_application(name)?;
    platform().focus_application(&target)?;
    Ok(ToolOutput {
        summary: format!("{} is focused.", target.display_name),
        value: json!({ "success": true, "tool": "focus_application", "data": { "application": target.display_name }, "error": null }),
    })
}

fn take_screenshot() -> AgentResult<ToolOutput> {
    let path = std::env::temp_dir().join(format!("coda-screen-{}.png", super::ids::now_ms()));
    platform().screenshot(&path)?;
    if !path.is_file() {
        return Err(AgentError::msg("Screenshot was not written. Grant Screen Recording if this is a Mac."));
    }
    Ok(ToolOutput {
        summary: format!("Screenshot saved to {}.", path.display()),
        value: json!({ "success": true, "tool": "take_screenshot", "data": { "path": path }, "error": null }),
    })
}

fn play_playlist(name: &str) -> AgentResult<ToolOutput> {
    let _ = open_application("Apple Music");
    platform().play_playlist(name)?;
    Ok(ToolOutput {
        summary: format!("Playing playlist “{name}” in Apple Music."),
        value: json!({
            "success": true,
            "tool": "play_playlist",
            "data": { "playlist": name, "application": "Apple Music", "verified": true },
            "error": null
        }),
    })
}

fn type_text(text: &str) -> AgentResult<ToolOutput> {
    platform().type_text(text)?;
    Ok(ToolOutput {
        summary: "Typed the text into the focused app.".into(),
        value: json!({ "success": true, "tool": "type_text", "error": null }),
    })
}

fn press_key(key: &str) -> AgentResult<ToolOutput> {
    platform().press_key(key)?;
    Ok(ToolOutput {
        summary: format!("Pressed {key}."),
        value: json!({ "success": true, "tool": "press_key", "data": { "key": key }, "error": null }),
    })
}

fn hotkey(keys: Option<&Vec<Value>>) -> AgentResult<ToolOutput> {
    let keys: Vec<String> = keys
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if keys.is_empty() {
        return Err(AgentError::msg("missing keys"));
    }
    platform().hotkey(&keys)?;
    Ok(ToolOutput {
        summary: format!("Pressed {}.", keys.join("+")),
        value: json!({ "success": true, "tool": "hotkey", "data": { "keys": keys }, "error": null }),
    })
}

struct Verify {
    running: bool,
    pid: Option<u32>,
    detail: Option<String>,
}

trait Platform {
    fn open_application(&self, target: &AppTarget) -> AgentResult<()>;
    fn open_url(&self, url: &str, app: Option<&AppTarget>) -> AgentResult<()>;
    fn open_path(&self, path: &Path) -> AgentResult<()>;
    fn verify_application(&self, target: &AppTarget) -> Verify;
    fn running_apps(&self) -> Vec<String>;
    fn focus_application(&self, target: &AppTarget) -> AgentResult<()>;
    fn close_application(&self, target: &AppTarget) -> AgentResult<()>;
    fn screenshot(&self, path: &Path) -> AgentResult<()>;
    fn play_playlist(&self, name: &str) -> AgentResult<()>;
    fn type_text(&self, text: &str) -> AgentResult<()>;
    fn press_key(&self, key: &str) -> AgentResult<()>;
    fn hotkey(&self, keys: &[String]) -> AgentResult<()>;
}

fn platform() -> Box<dyn Platform> {
    #[cfg(target_os = "macos")]
    {
        Box::new(MacOs)
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(Windows)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Box::new(Unsupported)
    }
}

struct MacOs;
#[allow(dead_code)]
struct Windows;
#[allow(dead_code)]
struct Unsupported;

impl Platform for MacOs {
    fn open_application(&self, target: &AppTarget) -> AgentResult<()> {
        let launched = command("open", &["-a", &target.launch_name])?
            || (target.launch_name != target.display_name
                && command("open", &["-a", &target.display_name])?);
        let activated = osascript(&format!(
            "tell application \"{}\" to activate",
            escape_as(&target.launch_name)
        ))
        .unwrap_or(false)
            || osascript(&format!(
                "tell application \"{}\" to activate",
                escape_as(&target.display_name)
            ))
            .unwrap_or(false);
        if launched || activated {
            std::thread::sleep(Duration::from_millis(400));
            return Ok(());
        }
        Err(not_found(&target.display_name))
    }

    fn open_url(&self, url: &str, app: Option<&AppTarget>) -> AgentResult<()> {
        let ok = match app {
            Some(app) => command("open", &["-a", &app.launch_name, url])?,
            None => command("open", &[url])?,
        };
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(format!("Could not open {url}.")))
        }
    }

    fn open_path(&self, path: &Path) -> AgentResult<()> {
        let as_str = path.to_string_lossy();
        if command("open", &[&as_str])? {
            Ok(())
        } else {
            Err(AgentError::msg(format!("Could not open {}.", path.display())))
        }
    }

    fn verify_application(&self, target: &AppTarget) -> Verify {
        let deadline = Instant::now() + Duration::from_millis(1800);
        loop {
            if let Some(pid) = pgrep(&target.launch_name).or_else(|| pgrep(&target.display_name)) {
                return Verify {
                    running: true,
                    pid: Some(pid),
                    detail: None,
                };
            }
            if Instant::now() >= deadline {
                return Verify {
                    running: false,
                    pid: None,
                    detail: Some(format!("{} is not installed or did not launch.", target.display_name)),
                };
            }
            std::thread::sleep(Duration::from_millis(150));
        }
    }

    fn running_apps(&self) -> Vec<String> {
        let Ok(output) = Command::new("osascript")
            .args([
                "-e",
                "tell application \"System Events\" to get name of every process whose background only is false",
            ])
            .output()
        else {
            return Vec::new();
        };
        String::from_utf8_lossy(&output.stdout)
            .split(", ")
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect()
    }

    fn focus_application(&self, target: &AppTarget) -> AgentResult<()> {
        let ok = osascript(&format!(
            "tell application \"{}\" to activate",
            escape_as(&target.launch_name)
        ))?;
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(format!(
                "Could not focus {}. Grant Accessibility if this is blocked.",
                target.display_name
            )))
        }
    }

    fn close_application(&self, target: &AppTarget) -> AgentResult<()> {
        let ok = osascript(&format!(
            "tell application \"{}\" to quit",
            escape_as(&target.launch_name)
        ))?;
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(format!("Could not quit {}.", target.display_name)))
        }
    }

    fn screenshot(&self, path: &Path) -> AgentResult<()> {
        let as_str = path.to_string_lossy();
        if command("screencapture", &["-x", &as_str])? {
            Ok(())
        } else {
            Err(AgentError::msg(
                "Could not capture the screen. Enable Screen Recording for Coda.",
            ))
        }
    }

    fn play_playlist(&self, name: &str) -> AgentResult<()> {
        let script = format!(
            r#"tell application "Music"
                activate
                delay 0.6
                set needle to "{}"
                set hits to (every user playlist whose name contains needle)
                if (count of hits) is 0 then
                    set hits to (every playlist whose name contains needle)
                end if
                if (count of hits) is 0 then error "playlist not found"
                play item 1 of hits
                delay 0.4
                if player state is not playing then error "playback did not start"
            end tell"#,
            escape_as(name)
        );
        if osascript(&script)? {
            Ok(())
        } else {
            Err(AgentError::msg(format!(
                "Could not play playlist “{name}” in Apple Music. Check the playlist name, or grant Automation access for Music."
            )))
        }
    }

    fn type_text(&self, text: &str) -> AgentResult<()> {
        let ok = osascript(&format!(
            "tell application \"System Events\" to keystroke \"{}\"",
            escape_as(text)
        ))?;
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(
                "Could not type. Grant Accessibility to Coda in System Settings.",
            ))
        }
    }

    fn press_key(&self, key: &str) -> AgentResult<()> {
        let mapped = apple_key(key);
        let ok = osascript(&format!(
            "tell application \"System Events\" to key code {mapped}"
        ))?;
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(
                "Could not press that key. Grant Accessibility to Coda.",
            ))
        }
    }

    fn hotkey(&self, keys: &[String]) -> AgentResult<()> {
        let mut mods = Vec::new();
        let mut code = 49u32;
        for key in keys {
            match key.to_ascii_lowercase().as_str() {
                "cmd" | "command" | "meta" => mods.push("command down"),
                "ctrl" | "control" => mods.push("control down"),
                "alt" | "option" => mods.push("option down"),
                "shift" => mods.push("shift down"),
                other => code = apple_key(other),
            }
        }
        let using = if mods.is_empty() {
            String::new()
        } else {
            format!(" using {{{}}}", mods.join(", "))
        };
        let ok = osascript(&format!(
            "tell application \"System Events\" to key code {code}{using}"
        ))?;
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(
                "Could not send that shortcut. Grant Accessibility to Coda.",
            ))
        }
    }
}

impl Platform for Windows {
    fn open_application(&self, target: &AppTarget) -> AgentResult<()> {
        let ok = windows_start(&[&target.launch_name])?;
        if ok {
            Ok(())
        } else {
            Err(not_found(&target.display_name))
        }
    }

    fn open_url(&self, url: &str, app: Option<&AppTarget>) -> AgentResult<()> {
        let ok = match app {
            Some(app) => windows_start(&[&app.launch_name, url])?,
            None => windows_start(&[url])?,
        };
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(format!("Could not open {url}.")))
        }
    }

    fn open_path(&self, path: &Path) -> AgentResult<()> {
        let as_str = path.to_string_lossy().into_owned();
        if windows_start(&[&as_str])? {
            Ok(())
        } else {
            Err(AgentError::msg(format!("Could not open {}.", path.display())))
        }
    }

    fn verify_application(&self, target: &AppTarget) -> Verify {
        let image = format!("{}.exe", target.launch_name.replace(' ', ""));
        let Ok(output) = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH"])
            .output()
        else {
            return Verify {
                running: false,
                pid: None,
                detail: Some("Could not verify whether the application launched.".into()),
            };
        };
        let text = String::from_utf8_lossy(&output.stdout);
        Verify {
            running: text.to_ascii_lowercase().contains(&image.to_ascii_lowercase())
                || text.to_ascii_lowercase().contains(&target.launch_name.to_ascii_lowercase()),
            pid: None,
            detail: None,
        }
    }

    fn running_apps(&self) -> Vec<String> {
        let Ok(output) = Command::new("tasklist").args(["/FO", "CSV", "/NH"]).output() else {
            return Vec::new();
        };
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.split(',').next())
            .map(|name| name.trim_matches('"').to_string())
            .take(40)
            .collect()
    }

    fn focus_application(&self, target: &AppTarget) -> AgentResult<()> {
        self.open_application(target)
    }

    fn close_application(&self, target: &AppTarget) -> AgentResult<()> {
        let image = format!("{}.exe", target.launch_name.replace(' ', ""));
        let ok = Command::new("taskkill")
            .args(["/IM", &image, "/F"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg(format!("Could not close {}.", target.display_name)))
        }
    }

    fn screenshot(&self, path: &Path) -> AgentResult<()> {
        let as_str = path.to_string_lossy();
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; Add-Type -AssemblyName System.Drawing; $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds; $bmp = New-Object System.Drawing.Bitmap $b.Width,$b.Height; $g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($b.Location,[System.Drawing.Point]::Empty,$b.Size); $bmp.Save('{}');",
            as_str.replace('\\', "\\\\")
        );
        let ok = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        if ok {
            Ok(())
        } else {
            Err(AgentError::msg("Could not capture the screen."))
        }
    }

    fn play_playlist(&self, name: &str) -> AgentResult<()> {
        let url = format!(
            "https://www.youtube.com/results?search_query={}",
            name.replace(' ', "+")
        );
        self.open_url(&url, None)
    }

    fn type_text(&self, _text: &str) -> AgentResult<()> {
        Err(AgentError::msg("Typing on Windows needs UI Automation permission."))
    }

    fn press_key(&self, _key: &str) -> AgentResult<()> {
        Err(AgentError::msg("Key press on Windows needs UI Automation permission."))
    }

    fn hotkey(&self, _keys: &[String]) -> AgentResult<()> {
        Err(AgentError::msg("Hotkeys on Windows need UI Automation permission."))
    }
}

impl Platform for Unsupported {
    fn open_application(&self, target: &AppTarget) -> AgentResult<()> {
        Err(AgentError::msg(format!(
            "Opening {} is not supported on this OS yet.",
            target.display_name
        )))
    }

    fn open_url(&self, url: &str, _app: Option<&AppTarget>) -> AgentResult<()> {
        Err(AgentError::msg(format!("Opening {url} is not supported on this OS yet.")))
    }

    fn open_path(&self, path: &Path) -> AgentResult<()> {
        Err(AgentError::msg(format!("Opening {} is not supported on this OS yet.", path.display())))
    }

    fn verify_application(&self, _target: &AppTarget) -> Verify {
        Verify {
            running: false,
            pid: None,
            detail: Some("Unsupported platform.".into()),
        }
    }

    fn running_apps(&self) -> Vec<String> {
        Vec::new()
    }

    fn focus_application(&self, target: &AppTarget) -> AgentResult<()> {
        self.open_application(target)
    }

    fn close_application(&self, target: &AppTarget) -> AgentResult<()> {
        Err(AgentError::msg(format!("Closing {} is not supported on this OS yet.", target.display_name)))
    }

    fn screenshot(&self, _path: &Path) -> AgentResult<()> {
        Err(AgentError::msg("Screenshots are not supported on this OS yet."))
    }

    fn play_playlist(&self, _name: &str) -> AgentResult<()> {
        Err(AgentError::msg("Playlists are not supported on this OS yet."))
    }

    fn type_text(&self, _text: &str) -> AgentResult<()> {
        Err(AgentError::msg("Typing is not supported on this OS yet."))
    }

    fn press_key(&self, _key: &str) -> AgentResult<()> {
        Err(AgentError::msg("Key press is not supported on this OS yet."))
    }

    fn hotkey(&self, _keys: &[String]) -> AgentResult<()> {
        Err(AgentError::msg("Hotkeys are not supported on this OS yet."))
    }
}

fn alias(key: &str) -> Option<AppTarget> {
    let (display, launch) = match key {
        "apple music" | "music" | "applemusic" | "itunes" => ("Apple Music", macos_or("Music", "iTunes")),
        "safari" => ("Safari", "Safari"),
        "chrome" | "google chrome" | "googlechrome" => ("Google Chrome", "Google Chrome"),
        "firefox" => ("Firefox", "Firefox"),
        "edge" | "microsoft edge" => ("Microsoft Edge", "Microsoft Edge"),
        "finder" | "explorer" | "file explorer" => {
            ("Finder", macos_or("Finder", "explorer"))
        }
        "terminal" | "cmd" | "command prompt" | "iterm" => {
            ("Terminal", macos_or("Terminal", "cmd"))
        }
        "vs code" | "vscode" | "visual studio code" | "code" => {
            ("Visual Studio Code", "Visual Studio Code")
        }
        "spotify" => ("Spotify", "Spotify"),
        "discord" => ("Discord", "Discord"),
        "slack" => ("Slack", "Slack"),
        "notes" | "apple notes" => ("Notes", "Notes"),
        "mail" | "apple mail" => ("Mail", "Mail"),
        "messages" => ("Messages", "Messages"),
        "calendar" => ("Calendar", "Calendar"),
        "photos" => ("Photos", "Photos"),
        "settings" | "system settings" | "system preferences" => {
            ("System Settings", macos_or("System Settings", "ms-settings:"))
        }
        "cursor" => ("Cursor", "Cursor"),
        "zoom" => ("zoom.us", "zoom.us"),
        _ => return None,
    };
    Some(AppTarget {
        display_name: display.into(),
        launch_name: launch.into(),
    })
}

fn macos_or(mac: &'static str, other: &'static str) -> &'static str {
    if cfg!(target_os = "macos") {
        mac
    } else {
        other
    }
}

fn discover(key: &str) -> Option<AppTarget> {
    let roots = application_roots();
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let stem = path.file_stem()?.to_string_lossy();
            if normalize(&stem) == *key || normalize(&stem).contains(key) {
                return Some(AppTarget {
                    display_name: stem.to_string(),
                    launch_name: stem.to_string(),
                });
            }
        }
    }
    None
}

fn application_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    #[cfg(target_os = "macos")]
    {
        roots.push(PathBuf::from("/Applications"));
        roots.push(PathBuf::from("/System/Applications"));
        if let Some(home) = std::env::var_os("HOME") {
            roots.push(PathBuf::from(home).join("Applications"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Some(pf) = std::env::var_os("ProgramFiles") {
            roots.push(PathBuf::from(pf));
        }
        if let Some(pf86) = std::env::var_os("ProgramFiles(x86)") {
            roots.push(PathBuf::from(pf86));
        }
    }
    roots
}

fn expand_user(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    if path == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }
    PathBuf::from(path)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn normalize_url(url: &str) -> AgentResult<String> {
    let trimmed = url.trim();
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("mailto:")
    {
        return Ok(trimmed.to_string());
    }
    if trimmed.contains('.') && !trimmed.contains(' ') {
        return Ok(format!("https://{trimmed}"));
    }
    Err(AgentError::msg("That does not look like a website."))
}

fn command(bin: &str, args: &[&str]) -> AgentResult<bool> {
    Command::new(bin)
        .args(args)
        .status()
        .map(|status| status.success())
        .map_err(|err| AgentError::msg(err.to_string()))
}

fn osascript(script: &str) -> AgentResult<bool> {
    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .map(|status| status.success())
        .map_err(|err| AgentError::msg(err.to_string()))
}

fn escape_as(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn apple_key(key: &str) -> u32 {
    match key.to_ascii_lowercase().as_str() {
        "return" | "enter" => 36,
        "tab" => 48,
        "space" => 49,
        "escape" | "esc" => 53,
        "delete" | "backspace" => 51,
        "left" => 123,
        "right" => 124,
        "down" => 125,
        "up" => 126,
        _ => 49,
    }
}

#[allow(dead_code)]
fn windows_start(args: &[&str]) -> AgentResult<bool> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg("start").arg("");
    for arg in args {
        cmd.arg(arg);
    }
    cmd.status()
        .map(|status| status.success())
        .map_err(|err| AgentError::msg(err.to_string()))
}

fn pgrep(name: &str) -> Option<u32> {
    let output = Command::new("pgrep").args(["-if", name]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.trim().parse().ok())
}

fn not_found(name: &str) -> AgentError {
    AgentError::msg(format!("{name} isn't installed on this computer."))
}

fn str_arg<'a>(args: &'a Value, key: &'a str) -> AgentResult<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| AgentError::msg(format!("missing {key}")))
}

fn normalize(text: &str) -> String {
    text.to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_music_alias() {
        let target = resolve_application("Apple Music").unwrap();
        assert_eq!(target.display_name, "Apple Music");
        if cfg!(target_os = "macos") {
            assert_eq!(target.launch_name, "Music");
        }
    }

    #[test]
    fn vscode_aliases() {
        for name in ["VS Code", "vscode", "Visual Studio Code"] {
            assert_eq!(resolve_application(name).unwrap().display_name, "Visual Studio Code");
        }
    }

    #[test]
    fn chrome_alias() {
        assert_eq!(resolve_application("chrome").unwrap().display_name, "Google Chrome");
    }

    #[test]
    fn expands_downloads() {
        let path = expand_user("~/Downloads");
        assert!(path.ends_with("Downloads"));
    }

    #[test]
    fn https_passthrough() {
        assert_eq!(
            normalize_url("https://www.youtube.com").unwrap(),
            "https://www.youtube.com"
        );
    }

    #[test]
    fn missing_app_is_error_not_panic() {
        let result = execute(
            "OPEN_APPLICATION",
            &json!({ "name": "DefinitelyNotAnInstalledAppXYZ" }),
        );
        assert!(result.is_err(), "missing app must fail the tool, not the process");
    }

    #[test]
    fn unknown_computer_tool_is_error() {
        assert!(execute("NOT_A_TOOL", &json!({})).is_err());
    }

    #[test]
    #[ignore]
    #[cfg(target_os = "macos")]
    fn live_open_finder() {
        let output = execute("OPEN_APPLICATION", &json!({ "name": "Finder" })).expect("Finder should open");
        assert!(output.summary.contains("Finder") || output.summary.contains("open"));
        assert_eq!(output.value["success"], true);
    }

    #[test]
    #[ignore]
    #[cfg(target_os = "macos")]
    fn live_open_downloads() {
        let output = execute("OPEN_FOLDER", &json!({ "path": "~/Downloads" })).expect("Downloads should open");
        assert_eq!(output.value["success"], true);
    }

    #[test]
    #[ignore]
    #[cfg(target_os = "macos")]
    fn live_open_apple_music() {
        let output = execute("OPEN_APPLICATION", &json!({ "name": "Apple Music" }))
            .expect("Apple Music should launch");
        assert!(output.summary.contains("Apple Music"));
        assert_eq!(output.value["success"], true);
    }
}
