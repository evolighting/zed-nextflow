use std::fs;

use zed_extension_api::{self as zed, Os, current_platform, process, settings::LspSettings};

const LANGUAGE_SERVER_ID: &str = "nextflow-language-server";
const DEFAULT_LANGUAGE_VERSION: &str = "26.04";
const UNIX_CACHE_MANAGER_PATH: &str = "nextflow-lsp-cache";
const UNIX_CACHE_MANAGER_SCRIPT: &str = include_str!("../scripts/nextflow-lsp-cache");
const WINDOWS_CACHE_MANAGER_PATH: &str = "nextflow-lsp-cache.ps1";
const WINDOWS_CACHE_MANAGER_SCRIPT: &str = include_str!("../scripts/nextflow-lsp-cache.ps1");

struct NextflowExtension;

fn default_workspace_configuration() -> zed::serde_json::Value {
    zed::serde_json::json!({
        "nextflow": {
            "languageVersion": DEFAULT_LANGUAGE_VERSION,
            "completion": {
                "extended": false,
                "maxItems": 100
            },
            "debug": false,
            "errorReportingMode": "warnings",
            "files": {
                "exclude": [
                    ".git",
                    ".lineage",
                    ".nf-test",
                    ".pixi",
                    ".venv",
                    "work"
                ]
            },
            "formatting": {
                "harshilAlignment": false,
                "maheshForm": false,
                "sortDeclarations": false
            }
        }
    })
}

fn merge_configuration(defaults: &mut zed::serde_json::Value, overrides: zed::serde_json::Value) {
    match (defaults, overrides) {
        (zed::serde_json::Value::Object(defaults), zed::serde_json::Value::Object(overrides)) => {
            for (key, value) in overrides {
                match defaults.get_mut(&key) {
                    Some(default) => merge_configuration(default, value),
                    None => {
                        defaults.insert(key, value);
                    }
                }
            }
        }
        (default, override_value) => *default = override_value,
    }
}

impl NextflowExtension {
    fn install_cache_manager(path: &str, contents: &str) -> zed::Result<()> {
        if fs::read(path).ok().as_deref() == Some(contents.as_bytes()) {
            return Ok(());
        }
        fs::write(path, contents)
            .map_err(|error| format!("failed to install cache manager {path}: {error}"))
    }

    fn windows_powershell(worktree: &zed::Worktree) -> zed::Result<&'static str> {
        // Use a stable command name rather than the absolute path returned by
        // which(), because process capabilities match the command string.
        ["pwsh", "pwsh.exe", "powershell", "powershell.exe"]
            .into_iter()
            .find(|command| worktree.which(command).is_some())
            .ok_or_else(|| {
                "PowerShell is required to manage the shared Nextflow language-server cache on Windows. Install PowerShell 7 (pwsh) or make Windows PowerShell (powershell.exe) available on PATH.".to_string()
            })
    }

    fn cache_manager_command(
        worktree: &zed::Worktree,
        language_version: &str,
    ) -> zed::Result<process::Command> {
        match current_platform().0 {
            Os::Windows => {
                Self::install_cache_manager(
                    WINDOWS_CACHE_MANAGER_PATH,
                    WINDOWS_CACHE_MANAGER_SCRIPT,
                )?;
                let command = Self::windows_powershell(worktree)?;
                Ok(process::Command::new(command)
                    .args([
                        "-NoLogo",
                        "-NoProfile",
                        "-NonInteractive",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        WINDOWS_CACHE_MANAGER_PATH,
                        "resolve",
                        language_version,
                    ])
                    .envs(worktree.shell_env()))
            }
            Os::Linux | Os::Mac => {
                // Normalize CRLF before an embedded script reaches a Unix shell.
                let script = UNIX_CACHE_MANAGER_SCRIPT.replace("\r\n", "\n");
                Self::install_cache_manager(UNIX_CACHE_MANAGER_PATH, &script)?;
                Ok(process::Command::new("bash")
                    .args([UNIX_CACHE_MANAGER_PATH, "resolve", language_version])
                    .envs(worktree.shell_env()))
            }
        }
    }

    fn configured_command(worktree: &zed::Worktree) -> zed::Result<Option<zed::Command>> {
        let Some(binary) = LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree)?.binary else {
            return Ok(None);
        };
        let Some(command) = binary.path else {
            if binary.arguments.is_some() || binary.env.is_some() {
                return Err(format!(
                    "lsp.{LANGUAGE_SERVER_ID}.binary.path is required when binary arguments or environment variables are configured"
                ));
            }
            return Ok(None);
        };

        Ok(Some(zed::Command {
            command,
            args: binary.arguments.unwrap_or_default(),
            env: binary.env.unwrap_or_default().into_iter().collect(),
        }))
    }

    fn normalize_language_version(value: &str) -> Option<String> {
        let value = value.trim().strip_prefix('v').unwrap_or(value.trim());
        let mut parts = value.split('.');
        let major = parts.next()?;
        let minor = parts.next()?;
        if parts.next().is_some()
            || major.is_empty()
            || minor.is_empty()
            || !major.bytes().all(|byte| byte.is_ascii_digit())
            || !minor.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        Some(format!("{major}.{minor}"))
    }

    fn language_version(worktree: &zed::Worktree) -> zed::Result<String> {
        let settings = LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree)?;
        let configured = settings
            .settings
            .as_ref()
            .and_then(|settings| settings.pointer("/nextflow/languageVersion"))
            .and_then(zed::serde_json::Value::as_str);

        match configured {
            Some(value) => Self::normalize_language_version(value).ok_or_else(|| {
                format!(
                    "lsp.{LANGUAGE_SERVER_ID}.settings.nextflow.languageVersion must look like \"26.04\""
                )
            }),
            None => Ok(DEFAULT_LANGUAGE_VERSION.to_string()),
        }
    }

    fn shared_cache_jar(
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        let language_version = Self::language_version(worktree)?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let output = Self::cache_manager_command(worktree, &language_version)?
            .output()
            .map_err(|error| format!("failed to resolve the Nextflow language server: {error}"))?;

        if output.status != Some(0) {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "failed to resolve Nextflow language server for {language_version}: {}",
                stderr.trim()
            ));
        }

        let jar_path = String::from_utf8(output.stdout)
            .map_err(|_| "Nextflow language-server cache returned a non-UTF-8 path".to_string())?;
        let jar_path = jar_path.trim();
        if jar_path.is_empty() {
            return Err(format!(
                "Nextflow language-server cache returned no JAR for {language_version}"
            ));
        }
        Ok(jar_path.to_string())
    }
}

impl zed::Extension for NextflowExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if let Some(command) = Self::configured_command(worktree)? {
            return Ok(command);
        }

        let java = worktree.which("java").ok_or_else(|| {
            format!(
                "Java 17 or later is required by the Nextflow language server. Install Java and make `java` available on PATH, or configure lsp.{LANGUAGE_SERVER_ID}.binary."
            )
        })?;
        let jar = Self::shared_cache_jar(language_server_id, worktree)?;

        Ok(zed::Command {
            command: java,
            args: vec!["-jar".to_string(), jar],
            env: worktree.shell_env(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(
            LspSettings::for_worktree(language_server_id.as_ref(), worktree)?
                .initialization_options,
        )
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?.settings;
        let mut configuration = default_workspace_configuration();
        if let Some(settings) = settings {
            if !settings.is_object() {
                return Err(format!(
                    "lsp.{LANGUAGE_SERVER_ID}.settings must be a JSON object"
                ));
            }
            merge_configuration(&mut configuration, settings);
        }
        Ok(Some(configuration))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_settings_are_merged_with_workspace_defaults() {
        let mut configuration = default_workspace_configuration();
        merge_configuration(
            &mut configuration,
            zed::serde_json::json!({
                "nextflow": {
                    "languageVersion": "25.10",
                    "completion": {
                        "extended": true
                    }
                }
            }),
        );

        assert_eq!(configuration["nextflow"]["languageVersion"], "25.10");
        assert_eq!(configuration["nextflow"]["completion"]["extended"], true);
        assert_eq!(configuration["nextflow"]["completion"]["maxItems"], 100);
        assert_eq!(
            configuration["nextflow"]["files"]["exclude"],
            zed::serde_json::json!([".git", ".lineage", ".nf-test", ".pixi", ".venv", "work"])
        );
    }

    #[test]
    fn user_arrays_replace_default_arrays() {
        let mut configuration = default_workspace_configuration();
        merge_configuration(
            &mut configuration,
            zed::serde_json::json!({
                "nextflow": {
                    "files": {
                        "exclude": ["build"]
                    }
                }
            }),
        );

        assert_eq!(
            configuration["nextflow"]["files"]["exclude"],
            zed::serde_json::json!(["build"])
        );
    }
}

zed::register_extension!(NextflowExtension);
