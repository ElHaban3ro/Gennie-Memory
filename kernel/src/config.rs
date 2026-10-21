use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_path: PathBuf,
    pub evaluation_capture_path: Option<PathBuf>,
    pub evaluation_response_content_path: Option<PathBuf>,
}

pub struct Secrets {
    pub openrouter_api_key: Option<String>,
}

fn load_env_file() -> Result<(), String> {
    if std::path::Path::new(".env").exists() {
        dotenvy::from_filename(".env").map_err(|e| format!("no se pudo leer .env: {e}"))?;
    }
    Ok(())
}

pub fn load() -> Result<Config, String> {
    load_env_file()?;
    let value = std::env::var("IROMEM_DATABASE_PATH")
        .map_err(|_| "falta IROMEM_DATABASE_PATH".to_string())?;
    if value.trim().is_empty() {
        return Err("IROMEM_DATABASE_PATH está vacío".to_string());
    }
    Ok(Config {
        database_path: PathBuf::from(value),
        evaluation_capture_path: read_evaluation_capture_path(|name| std::env::var(name))?,
        evaluation_response_content_path: read_evaluation_response_content_path(|name| {
            std::env::var(name)
        })?,
    })
}

fn read_evaluation_response_content_path(
    get: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Result<Option<PathBuf>, String> {
    match get("IROMEM_EVALUATION_RESPONSE_CONTENT_PATH") {
        Ok(value) if value.trim().is_empty() => {
            Err("IROMEM_EVALUATION_RESPONSE_CONTENT_PATH está vacío".into())
        }
        Ok(value) => Ok(Some(PathBuf::from(value))),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err("IROMEM_EVALUATION_RESPONSE_CONTENT_PATH no es una ruta válida".into())
        }
    }
}

fn read_evaluation_capture_path(
    get: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Result<Option<PathBuf>, String> {
    match get("IROMEM_EVALUATION_CAPTURE_PATH") {
        Ok(value) if value.trim().is_empty() => {
            Err("IROMEM_EVALUATION_CAPTURE_PATH está vacío".into())
        }
        Ok(value) => Ok(Some(PathBuf::from(value))),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err("IROMEM_EVALUATION_CAPTURE_PATH no es una ruta válida".into())
        }
    }
}

pub fn load_secrets() -> Result<Secrets, String> {
    load_env_file()?;
    Ok(Secrets {
        openrouter_api_key: read_openrouter_key(|name| std::env::var(name).ok()),
    })
}

fn read_openrouter_key(get: impl Fn(&str) -> Option<String>) -> Option<String> {
    get("IROMEM_OPENROUTER_API_KEY").filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_iromem_openrouter_variable_name() {
        let key = read_openrouter_key(|name| {
            if name == "IROMEM_OPENROUTER_API_KEY" {
                Some("synthetic-test-token".into())
            } else {
                None
            }
        });
        assert_eq!(key.as_deref(), Some("synthetic-test-token"));
        assert!(read_openrouter_key(|_| Some("  ".into())).is_none());
    }

    #[test]
    fn evaluation_capture_path_is_optional_and_rejects_empty_values() {
        assert_eq!(
            read_evaluation_capture_path(|_| Err(std::env::VarError::NotPresent)).unwrap(),
            None
        );
        assert_eq!(
            read_evaluation_capture_path(|_| Ok("/tmp/iromem-capture.jsonl".into())).unwrap(),
            Some(PathBuf::from("/tmp/iromem-capture.jsonl"))
        );
        assert!(read_evaluation_capture_path(|_| Ok("  ".into())).is_err());
    }

    #[test]
    fn evaluation_response_content_path_is_separately_opt_in() {
        assert_eq!(
            read_evaluation_response_content_path(|_| Err(std::env::VarError::NotPresent)).unwrap(),
            None
        );
        assert_eq!(
            read_evaluation_response_content_path(|_| Ok("/tmp/iromem-response.jsonl".into()))
                .unwrap(),
            Some(PathBuf::from("/tmp/iromem-response.jsonl"))
        );
        assert!(read_evaluation_response_content_path(|_| Ok(" ".into())).is_err());
    }
}
