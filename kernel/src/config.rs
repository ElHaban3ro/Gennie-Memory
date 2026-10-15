use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_path: PathBuf,
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
    })
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
}
