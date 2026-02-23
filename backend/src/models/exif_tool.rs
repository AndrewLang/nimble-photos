use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use serde_json::Value;

pub type ExifMap = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct ExifTool {
    exe_path: PathBuf,
}

impl ExifTool {
    pub fn new() -> Self {
        Self {
            exe_path: Self::default_exe_path(),
        }
    }

    pub fn read_exif(&self, path: &str) -> Result<ExifMap> {
        if !self.exe_path.exists() {
            anyhow::bail!("ExifTool binary not found at {:?}", self.exe_path);
        }

        let output = Command::new(&self.exe_path)
            .arg("-json")
            .arg("-n") // numeric values (clean)
            .arg("-fast") // good speed
            .arg(path)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "ExifTool failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let json = String::from_utf8_lossy(&output.stdout);
        let parsed: Vec<Value> = serde_json::from_str(&json)?;

        if parsed.is_empty() {
            anyhow::bail!("ExifTool returned no data");
        }

        Ok(Self::json_to_map(&parsed[0]))
    }

    fn json_to_map(value: &Value) -> ExifMap {
        let mut map = HashMap::new();

        if let Value::Object(obj) = value {
            for (k, v) in obj {
                map.insert(k.clone(), Self::value_to_string(v));
            }
        }

        if let Some(height) = map.get("ImageHeight").cloned() {
            map.entry("ImageLength".to_string()).or_insert(height);
        }

        map
    }

    fn value_to_string(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Null => String::new(),
            _ => v.to_string(), // fallback for arrays/objects
        }
    }

    fn default_exe_path() -> PathBuf {
        let exe_name = if cfg!(target_os = "windows") {
            "exiftool.exe"
        } else {
            "exiftool"
        };

        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("bins")
            .join(exe_name)
    }
}
