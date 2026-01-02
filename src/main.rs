use chrono::{DateTime, Local};
use clap::Parser;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "notify-reg")]
#[command(about = "Automatically save desktop notifications to a file")]
struct Cli {
    #[arg(short = 'p', long, default_value = "./notifications.txt")]
    path: PathBuf,

    #[arg(short = 'c', long, default_value = "86400")]
    cleanup_interval: u64,
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path;
    let cleanup_interval = cli.cleanup_interval;

    let notifications = Arc::new(Mutex::new(Vec::new()));
    let notifications_cleanup = notifications.clone();
    let path_cleanup = path.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(60));
            cleanup_notifications(&notifications_cleanup, &path_cleanup, cleanup_interval);
        }
    });

    println!("Listening for notifications. Press Ctrl+C to exit.");
    println!("Saving to: {}", path.display());
    println!("Cleanup interval: {} seconds", cleanup_interval);

    if let Err(e) = listen_to_notifications(notifications, path) {
        eprintln!("Error: {}", e);
    }
}

fn listen_to_notifications(
    notifications: Arc<Mutex<Vec<NotificationEntry>>>,
    path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let child = Command::new("dbus-monitor")
        .arg("--session")
        .arg("interface='org.freedesktop.Notifications',member=Notify")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = child.stdout.ok_or("Failed to capture stdout")?;

    use std::io::{BufRead, BufReader};
    let reader = BufReader::new(stdout);

    let mut in_notification = false;
    let mut buffer = String::new();

    println!("开始监听通知...");

    for line_result in reader.lines() {
        let line = line_result?;

        if line.contains("method call") && line.contains("Notify") {
            in_notification = true;
            buffer.clear();
            buffer.push_str(&line);
        } else if in_notification {
            buffer.push_str("\n");
            buffer.push_str(&line);

            if line.trim().starts_with("int32") {
                in_notification = false;
                let entry = parse_notification(&buffer);
                {
                    let mut notifs = notifications.lock().unwrap();
                    notifs.push(entry.clone());
                }
                save_single_notification(&entry, &path);
                println!("保存: [{}] 标题: {}", entry.app_name, entry.summary);
            }
        }
    }

    Ok(())
}

fn parse_notification(text: &str) -> NotificationEntry {
    let lines: Vec<&str> = text.lines().collect();

    let app_name = if lines.len() > 1 {
        extract_quoted_string(lines[1])
    } else {
        "未知应用".to_string()
    };

    let summary = if lines.len() > 4 {
        extract_quoted_string(lines[4])
    } else {
        "无标题".to_string()
    };

    let body = if lines.len() > 5 {
        extract_quoted_string(lines[5])
    } else {
        "无内容".to_string()
    };

    NotificationEntry {
        timestamp: Local::now(),
        app_name,
        summary,
        body,
    }
}

fn extract_quoted_string(line: &str) -> String {
    let start = match line.find('"') {
        Some(pos) => pos + 1,
        None => return "无".to_string(),
    };

    if start >= line.len() {
        return "无".to_string();
    }

    let end = match line[start..].find('"') {
        Some(pos) => start + pos,
        None => return "无".to_string(),
    };

    if end <= start {
        return "无".to_string();
    }

    line[start..end].to_string()
}

fn cleanup_notifications(
    notifications: &Arc<Mutex<Vec<NotificationEntry>>>,
    path: &PathBuf,
    interval: u64,
) {
    let mut notifs = notifications.lock().unwrap();
    let now = Local::now();
    let before_count = notifs.len();

    notifs.retain(|n| now.signed_duration_since(n.timestamp).num_seconds() < interval as i64);

    let removed = before_count - notifs.len();
    if removed > 0 {
        save_all_notifications(&notifs, path);
        println!("清理了 {} 条旧通知", removed);
    }
}

fn save_single_notification(notification: &NotificationEntry, path: &PathBuf) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let line = format!(
            // "[{}] {}\n标题: \n{}\n内容: \n{}\n{}\n",
            "[{}] {}\n{}\n{}\n{}\n",
            notification.timestamp.format("%Y-%m-%d %H:%M:%S"),
            notification.app_name,
            notification.summary,
            notification.body,
            "=".repeat(50)
        );
        let _ = file.write_all(line.as_bytes());
    }
}

fn save_all_notifications(notifications: &[NotificationEntry], path: &PathBuf) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
    {
        for notification in notifications {
            let line = format!(
                // "[{}] {}\n标题: \n{}\n内容: \n{}\n{}\n",
                "[{}] {}\n{}\n{}\n{}\n",
                notification.timestamp.format("%Y-%m-%d %H:%M:%S"),
                notification.app_name,
                notification.summary,
                notification.body,
                "=".repeat(50)
            );
            let _ = file.write_all(line.as_bytes());
        }
    }
}

#[derive(Debug, Clone)]
struct NotificationEntry {
    timestamp: DateTime<Local>,
    app_name: String,
    summary: String,
    body: String,
}
