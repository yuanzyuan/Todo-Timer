use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct CompletedExportTask {
  title: String,
  #[serde(rename = "completedMarkdown")]
  completed_markdown: String,
  #[serde(rename = "completedAt")]
  completed_at: Option<i64>,
}

fn sanitize_filename(input: &str) -> String {
  let sanitized: String = input
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        c
      } else {
        '_'
      }
    })
    .collect();

  let trimmed = sanitized.trim_matches('_');
  if trimmed.is_empty() {
    "task".to_string()
  } else {
    trimmed.to_string()
  }
}

fn now_unix_seconds() -> i64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0)
}

#[tauri::command]
fn export_completed_markdowns(tasks: Vec<CompletedExportTask>, export_path: Option<String>) -> Result<String, String> {
  if tasks.is_empty() {
    return Err("没有可导出的已完成任务".to_string());
  }

  let export_dir = match export_path {
    Some(path) if !path.trim().is_empty() => {
      let trimmed = path.trim();
      if let Some(stripped) = trimmed.strip_prefix("~/") {
        let home = std::env::var("HOME").map_err(|_| "无法读取 HOME 目录".to_string())?;
        PathBuf::from(home).join(stripped)
      } else {
        Path::new(trimmed).to_path_buf()
      }
    }
    _ => {
      let home = std::env::var("HOME").map_err(|_| "无法读取 HOME 目录".to_string())?;
      PathBuf::from(home).join("Documents").join("todo-timer-exports")
    }
  };

  if !export_dir.is_absolute() {
    return Err("导出路径必须为绝对路径".to_string());
  }

  fs::create_dir_all(&export_dir).map_err(|e| format!("创建导出目录失败: {e}"))?;

  for (index, task) in tasks.iter().enumerate() {
    let title_part = sanitize_filename(&task.title);
    let time_part = task.completed_at.unwrap_or_else(now_unix_seconds);
    let file_name = format!("{:03}_{}_{}.md", index + 1, time_part, title_part);
    let file_path = export_dir.join(file_name);
    fs::write(&file_path, task.completed_markdown.as_bytes())
      .map_err(|e| format!("写入文件失败({}): {e}", file_path.display()))?;
  }

  Ok(export_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn choose_export_folder() -> Result<Option<String>, String> {
  #[cfg(target_os = "macos")]
  {
    let script = r#"set chosenFolder to choose folder with prompt "选择 Markdown 导出目录"
POSIX path of chosenFolder"#;
    let output = Command::new("osascript")
      .arg("-e")
      .arg(script)
      .output()
      .map_err(|e| format!("无法打开访达目录选择器: {e}"))?;

    if output.status.success() {
      let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
      if path.is_empty() {
        Ok(None)
      } else {
        Ok(Some(path))
      }
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
      // 用户点击取消时通常是 -128 / User canceled
      if stderr.contains("-128") || stderr.contains("User canceled") {
        Ok(None)
      } else {
        Err(format!("打开访达目录选择器失败: {stderr}"))
      }
    }
  }

  #[cfg(not(target_os = "macos"))]
  {
    Err("当前仅支持在 macOS 上弹出访达目录选择器".to_string())
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![export_completed_markdowns, choose_export_folder])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
