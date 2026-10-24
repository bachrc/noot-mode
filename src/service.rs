use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

pub const UNIT_NAME: &str = "nootd.service";
const UNIT_TEMPLATE: &str = include_str!("../packaging/nootd.service");
const EXEC_PLACEHOLDER: &str = "@EXEC@";

#[derive(Debug, Clone)]
pub struct ServiceCli {
    program: OsString,
    unit: &'static str,
    unit_dir: Option<PathBuf>,
}

impl Default for ServiceCli {
    fn default() -> Self {
        Self::new("systemctl")
    }
}

impl ServiceCli {
    #[must_use]
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            unit: UNIT_NAME,
            unit_dir: None,
        }
    }

    #[must_use]
    pub fn with_unit_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.unit_dir = Some(dir.into());
        self
    }

    pub fn unit_path(&self) -> Result<PathBuf> {
        let dir = match &self.unit_dir {
            Some(dir) => dir.clone(),
            None => config_home()?.join("systemd").join("user"),
        };
        Ok(dir.join(self.unit))
    }

    pub fn install_unit(&self, daemon: &Path) -> Result<()> {
        let path = self.unit_path()?;
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).with_context(|| format!("création de {}", parent.display()))?;
            }
            let content = UNIT_TEMPLATE.replace(EXEC_PLACEHOLDER, &daemon.display().to_string());
            std::fs::write(&path, content).with_context(|| format!("écriture de {}", path.display()))?;
        }
        self.reload()
    }

    pub fn reload(&self) -> Result<()> {
        let output = self
            .invoke(&["--user", "daemon-reload"])
            .with_context(|| format!("impossible d'exécuter {}", self.program.to_string_lossy()))?;
        if !output.status.success() {
            bail!(
                "systemctl daemon-reload a échoué : {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }

    pub fn is_active(&self) -> Result<bool> {
        let output = self
            .invoke(&["--user", "is-active", "--quiet", self.unit])
            .with_context(|| format!("impossible d'exécuter {}", self.program.to_string_lossy()))?;
        Ok(output.status.success())
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let action = if enabled { "enable" } else { "disable" };
        let output = self
            .invoke(&["--user", action, "--now", self.unit])
            .with_context(|| format!("impossible d'exécuter {}", self.program.to_string_lossy()))?;
        if !output.status.success() {
            bail!("systemctl {action} a échoué : {}", String::from_utf8_lossy(&output.stderr).trim());
        }
        Ok(())
    }

    pub fn toggle(&self) -> Result<bool> {
        let enabled = !self.is_active()?;
        self.set_enabled(enabled)?;
        Ok(enabled)
    }

    fn invoke(&self, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new(&self.program).args(args).output()
    }
}

fn config_home() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir));
    }
    let home = std::env::var_os("HOME").context("HOME absent de l'environnement")?;
    Ok(PathBuf::from(home).join(".config"))
}
