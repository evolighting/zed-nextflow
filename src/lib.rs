use zed_extension_api::{self as zed, process, settings::LspSettings};

const LANGUAGE_SERVER_ID: &str = "nextflow-language-server";
const DEFAULT_LANGUAGE_VERSION: &str = "26.04";
const CACHE_MANAGER_COMMAND: &str = "bash";
const CACHE_MANAGER_SCRIPT: &str = include_str!("../scripts/nextflow-lsp-cache");

struct NextflowExtension;

impl NextflowExtension {
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

        // Keep this command name in sync with the process:exec capability in extension.toml.
        // Passing the path returned by worktree.which() would turn this into e.g.
        // /usr/bin/bash, which is a different capability command.
        let output = process::Command::new(CACHE_MANAGER_COMMAND)
            .args([
                "-c",
                CACHE_MANAGER_SCRIPT,
                "nextflow-lsp-cache",
                "resolve",
                &language_version,
            ])
            .envs(worktree.shell_env())
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
        Ok(Some(settings.unwrap_or_else(|| {
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
        })))
    }
}

zed::register_extension!(NextflowExtension);
