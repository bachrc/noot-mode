use std::fs;
use std::io::ErrorKind;

use anyhow::{Context, Result};
use evdev::Device;

use crate::keys::is_keyboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputStatus {
    Ready,
    Blocked,
    NoKeyboard,
}

pub fn probe() -> Result<InputStatus> {
    let mut readable_keyboards = 0;
    let mut blocked = 0;

    for entry in fs::read_dir("/dev/input").context("lecture de /dev/input")? {
        let path = entry?.path();
        let is_event_node = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("event"));
        if !is_event_node {
            continue;
        }
        match Device::open(&path) {
            Ok(device) => {
                if is_keyboard(&device) {
                    readable_keyboards += 1;
                }
            }
            Err(error) if error.kind() == ErrorKind::PermissionDenied => blocked += 1,
            Err(_) => {}
        }
    }

    Ok(classify(readable_keyboards, blocked))
}

#[must_use]
pub const fn classify(readable_keyboards: usize, blocked: usize) -> InputStatus {
    if readable_keyboards > 0 {
        InputStatus::Ready
    } else if blocked > 0 {
        InputStatus::Blocked
    } else {
        InputStatus::NoKeyboard
    }
}
