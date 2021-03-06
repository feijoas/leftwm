use super::*;
use crate::display_action::DisplayAction;
use crate::utils::helpers;

pub fn process(manager: &mut Manager, command: Command, val: Option<String>) -> bool {
    match command {
        Command::MoveToTag if val.is_none() => false,
        Command::MoveToTag if !is_num(&val) => false,
        Command::MoveToTag if to_num(&val) > manager.tags.len() => false,
        Command::MoveToTag if to_num(&val) < 1 => false,
        Command::MoveToTag => {
            let tag_num = to_num(&val);
            let tag = manager.tags[tag_num - 1].clone();
            if let Some(window) = manager.focused_window_mut() {
                window.clear_tags();
                window.set_floating(false);
                window.tag(tag);
                return true;
            }
            false
        }

        Command::GotoTag if val.is_none() => false,
        Command::GotoTag if !is_num(&val) => false,
        Command::GotoTag => goto_tag_handler::process(manager, to_num(&val)),

        Command::Execute if val.is_none() => false,
        Command::Execute => {
            use std::process::{Command, Stdio};
            let _ = Command::new("sh")
                .arg("-c")
                .arg(&val.unwrap())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .spawn();
            false
        }

        Command::CloseWindow => {
            if let Some(window) = manager.focused_window() {
                let act = DisplayAction::KillWindow(window.handle.clone());
                manager.actions.push_back(act);
            }
            false
        }

        Command::SwapTags => {
            if manager.workspaces.len() >= 2 && manager.focused_workspace_history.len() >= 2 {
                let mut a = manager.workspaces[manager.focused_workspace_history[0]].clone();
                let mut b = manager.workspaces[manager.focused_workspace_history[1]].clone();
                let swap = a.tags.clone();
                a.tags = b.tags.clone();
                b.tags = swap;
                manager.workspaces[manager.focused_workspace_history[0]] = a;
                manager.workspaces[manager.focused_workspace_history[1]] = b;
                return true;
            }
            false
        }

        Command::MoveToLastWorkspace => {
            if manager.workspaces.len() >= 2 && manager.focused_workspace_history.len() >= 2 {
                let wp_tags = &manager.workspaces[manager.focused_workspace_history[1]]
                    .tags
                    .clone();
                if let Some(window) = manager.focused_window_mut() {
                    window.tags = vec![wp_tags[0].clone()];
                    return true;
                }
            }
            false
        }

        Command::NextLayout => {
            if let Some(workspace) = manager.focused_workspace_mut() {
                workspace.next_layout();
                return true;
            }
            false
        }
        Command::PreviousLayout => {
            if let Some(workspace) = manager.focused_workspace_mut() {
                workspace.prev_layout();
                return true;
            }
            false
        }

        Command::MoveWindowUp => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            helpers::reorder_vec(&mut to_reorder, is_handle, -1);
            manager.windows.append(&mut to_reorder);
            let act = DisplayAction::MoveMouseOver(handle);
            manager.actions.push_back(act);
            true
        }

        Command::MoveWindowDown => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut to_reorder = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            helpers::reorder_vec(&mut to_reorder, is_handle, 1);
            manager.windows.append(&mut to_reorder);
            let act = DisplayAction::MoveMouseOver(handle);
            manager.actions.push_back(act);
            true
        }

        Command::FocusWindowUp => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut window_group = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            if let Some(new_focused) = helpers::relative_find(&window_group, is_handle, -1) {
                let act = DisplayAction::MoveMouseOver(new_focused.handle.clone());
                manager.actions.push_back(act);
            }
            manager.windows.append(&mut window_group);
            true
        }

        Command::FocusWindowDown => {
            let handle = match manager.focused_window() {
                Some(h) => h.handle.clone(),
                _ => {
                    return false;
                }
            };
            let tags = match manager.focused_workspace() {
                Some(w) => w.tags.clone(),
                _ => {
                    return false;
                }
            };
            let for_active_workspace = |x: &Window| -> bool {
                helpers::intersect(&tags, &x.tags) && x.type_ != WindowType::Dock
            };
            let mut window_group = helpers::vec_extract(&mut manager.windows, for_active_workspace);
            let is_handle = |x: &Window| -> bool { x.handle == handle };
            if let Some(new_focused) = helpers::relative_find(&window_group, is_handle, 1) {
                let act = DisplayAction::MoveMouseOver(new_focused.handle.clone());
                manager.actions.push_back(act);
            }
            manager.windows.append(&mut window_group);
            true
        }

        Command::FocusWorkspaceNext => {
            let current = manager.focused_workspace();
            if current.is_none() {
                return false;
            }
            let current = current.unwrap();
            let mut index = match manager
                .workspaces
                .iter()
                .enumerate()
                .find(|&x| x.1 == current)
            {
                Some(x) => x.0,
                None => {
                    return false;
                }
            };
            index += 1;
            if index >= manager.workspaces.len() {
                index = 0;
            }
            let workspace = manager.workspaces[index].clone();
            focus_handler::focus_workspace(manager, &workspace);
            if let Some(window) = manager.windows.iter().find(|w| workspace.is_displaying(w)) {
                let window = window.clone();
                focus_handler::focus_window(manager, &window, &window.x() + 1, &window.y() + 1);
                let act = DisplayAction::MoveMouseOver(window.handle);
                manager.actions.push_back(act);
            }
            true
        }

        Command::FocusWorkspacePrevious => {
            let current = manager.focused_workspace();
            if current.is_none() {
                return false;
            }
            let current = current.unwrap();
            let mut index = match manager
                .workspaces
                .iter()
                .enumerate()
                .find(|&x| x.1 == current)
            {
                Some(x) => x.0 as i32,
                None => {
                    return false;
                }
            };
            index -= 1;
            if index < 0 {
                index = (manager.workspaces.len() as i32) - 1;
            }
            let workspace = manager.workspaces[index as usize].clone();
            focus_handler::focus_workspace(manager, &workspace);
            if let Some(window) = manager.windows.iter().find(|w| workspace.is_displaying(w)) {
                let window = window.clone();
                focus_handler::focus_window(manager, &window, &window.x() + 1, &window.y() + 1);
                let act = DisplayAction::MoveMouseOver(window.handle);
                manager.actions.push_back(act);
            }
            true
        }

        Command::MouseMoveWindow => false,

        Command::SoftReload => {
            let state_data = serde_json::to_string(&manager).unwrap();
            let _ = std::fs::write("/tmp/leftwm.state", state_data);
            ::std::process::exit(0);
        }
        Command::HardReload => ::std::process::exit(0),
    }
}

/// Is the string passed in a valid number
fn is_num(val: &Option<String>) -> bool {
    match val {
        Some(num) => num.parse::<usize>().is_ok(),
        None => false,
    }
}

/// Convert the option string to a number
fn to_num(val: &Option<String>) -> usize {
    val.as_ref()
        .and_then(|num| num.parse::<usize>().ok())
        .unwrap_or_default()
}
