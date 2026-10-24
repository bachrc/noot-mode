#![allow(clippy::unwrap_used, reason = "panics are only allowed in tests")]

use std::fs;
use std::os::unix::fs::PermissionsExt;

use googletest::prelude::*;
use noot::service::ServiceCli;

const STUB: &str = "#!/bin/sh\ndir=$(dirname \"$0\")\nprintf '%s\\n' \"$*\" >> \"$dir/calls.log\"\ncase \"$2\" in\n  is-active) [ -f \"$dir/active\" ] && exit 0 || exit 3 ;;\n  enable) : > \"$dir/active\" ;;\n  disable) rm -f \"$dir/active\" ;;\nesac\nexit 0\n";

fn stub() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("systemctl");
    fs::write(&path, STUB).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
    dir
}

fn calls(dir: &tempfile::TempDir) -> Vec<String> {
    let path = dir.path().join("calls.log");
    fs::read_to_string(path).unwrap().lines().map(str::to_owned).collect()
}

#[test]
fn is_active_follows_the_stub_state() -> anyhow::Result<()> {
    let dir = stub();
    let cli = ServiceCli::new(dir.path().join("systemctl"));

    assert_that!(cli.is_active()?, eq(false));

    fs::write(dir.path().join("active"), "").unwrap();
    assert_that!(cli.is_active()?, eq(true));

    Ok(())
}

#[test]
fn set_enabled_calls_systemctl_with_the_unit() -> anyhow::Result<()> {
    let dir = stub();
    let cli = ServiceCli::new(dir.path().join("systemctl"));

    cli.set_enabled(true)?;
    cli.set_enabled(false)?;

    assert_that!(
        calls(&dir),
        elements_are![eq("--user enable --now nootd.service"), eq("--user disable --now nootd.service")]
    );

    Ok(())
}

#[test]
fn toggle_flips_the_state() -> anyhow::Result<()> {
    let dir = stub();
    let cli = ServiceCli::new(dir.path().join("systemctl"));

    assert_that!(cli.toggle()?, eq(true));
    assert_that!(cli.is_active()?, eq(true));
    assert_that!(cli.toggle()?, eq(false));
    assert_that!(cli.is_active()?, eq(false));

    Ok(())
}

#[test]
fn failure_reports_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("systemctl");
    fs::write(&path, "#!/bin/sh\necho 'Unit nootd.service does not exist.' >&2\nexit 1\n").unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();

    let cli = ServiceCli::new(path);
    let error = cli.set_enabled(true).unwrap_err();

    assert_that!(format!("{error:#}"), contains_substring("does not exist"));
}
