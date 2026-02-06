use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub const DB_PATH: &str = ".stocks.json";
const DB_PATH_ENV: &str = "RUST_STOCK_DB_PATH";

#[derive(Serialize, Deserialize)]
struct StorageItem {
    code: String,
}

#[derive(Serialize, Deserialize)]
struct StorageData {
    stocks: Vec<StorageItem>,
}

fn get_db_path() -> DynResult<PathBuf> {
    if let Ok(path) = env::var(DB_PATH_ENV) {
        return Ok(PathBuf::from(path));
    }

    match dirs_next::home_dir() {
        Some(home) => Ok(home.join(DB_PATH)),
        None => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "home directory not found",
        ))),
    }
}

pub fn save_stocks(codes: &[String]) -> DynResult<()> {
    let db_path = get_db_path()?;
    let items: Vec<StorageItem> = codes
        .iter()
        .map(|c| StorageItem { code: c.clone() })
        .collect();

    let data = StorageData { stocks: items };
    let json = serde_json::to_string(&data)?;
    fs::write(db_path, json)?;
    Ok(())
}

pub fn load_stocks() -> DynResult<Vec<String>> {
    let db_path = get_db_path()?;
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(db_path)?;
    let data: StorageData = serde_json::from_str(&content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid storage data: {}", e),
        )
    })?;

    Ok(data.stocks.into_iter().map(|s| s.code).collect())
}

#[cfg(test)]
mod tests {
    use super::{load_stocks, save_stocks, DB_PATH_ENV};
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        env::temp_dir().join(format!("{}_{}.json", name, nanos))
    }

    #[test]
    fn storage_save_and_load_roundtrip() {
        let _guard = TEST_LOCK.lock().unwrap();
        let path = temp_path("stocks_roundtrip");
        env::set_var(DB_PATH_ENV, &path);

        let input = vec!["600519".to_string(), "NVDA".to_string()];
        save_stocks(&input).unwrap();
        let output = load_stocks().unwrap();

        env::remove_var(DB_PATH_ENV);
        let _ = fs::remove_file(&path);

        assert_eq!(input, output);
    }

    #[test]
    fn storage_invalid_json_returns_error() {
        let _guard = TEST_LOCK.lock().unwrap();
        let path = temp_path("stocks_invalid");
        env::set_var(DB_PATH_ENV, &path);

        fs::write(&path, "{not valid json").unwrap();
        let result = load_stocks();

        env::remove_var(DB_PATH_ENV);
        let _ = fs::remove_file(&path);

        assert!(result.is_err());
    }
}
