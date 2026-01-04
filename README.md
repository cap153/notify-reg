# Notify Reg - 通知记录工具

自动将 Linux 桌面通知保存到文本的 Rust 工具，支持定期清理。

## 功能特性

- 自动监听并保存桌面通知到文件
- 支持自定义存储路径
- 支持设置自动清理间隔
- 记录通知的时间戳、应用名称、标题和内容

## 📦 安装

### 方式一：源码编译 & 自动部署 (推荐)

如果您安装了 Rust 工具链，Makefile 会自动完成编译、安装以及 **Systemd 服务的生成与注册**。

```bash
git clone https://github.com/cap153/qb-floccus.git
cd qb-floccus

# 编译并安装到 ~/.local/bin，同时自动配置 Systemd
make install

systemctl --user daemon-reload
# 启动服务并配置开机自启动
systemctl --user enable --now qb-floccus
```

### 方式二：下载二进制文件 (手动部署)

1.  从 [Releases](https://github.com/cap153/qb-floccus/releases) 下载二进制文件并放入 PATH（如 `~/.local/bin/`）。
2.  **手动配置 [Systemd](开机自启动（systemd）)**

## 使用方法

### 基本使用

使用默认设置运行（保存到 `./notifications.txt`，24小时清理一次）：

```bash
notify-reg
```

### 自定义路径和清理时间

```bash
# 使用短参数
notify-reg -p /path/to/notifications.txt -c 3600
# 使用长参数
notify-reg --path /path/to/notifications.txt --cleanup-interval 3600
# 混合使用
notify-reg -p /tmp/notifications.txt` --cleanup-interval 3600
```

参数说明：
- `--path` 或 `-p`：指定通知保存的文件路径（默认：`./notifications.txt`）
- `--cleanup-interval` 或 `-c`：设置自动清理间隔，单位为秒（默认：86400秒 = 24小时）

### 作为后台服务运行

```bash
nohup notify-reg --path ~/notifications.txt --cleanup-interval 86400 &
```

### 开机自启动（systemd）

创建服务文件 `~/.config/systemd/user/notify-reg.service`：

```ini
[Unit]
Description=Notification Registry Service
After=dbus-session.service

[Service]
Type=simple
ExecStart=%h/.local/bin/notify-reg --path %h/notifications.txt --cleanup-interval 86400
Restart=always

[Install]
WantedBy=default.target
```

启用并启动服务：

```bash
systemctl --user daemon-reload
systemctl --user enable --now notify-reg
```

## 依赖

- Linux 桌面环境（如 Cosmic、GNOME、KDE 等）
- D-Bus 会话总线
- dbus-monitor 命令

## 注意事项

- 需要确保 D-Bus 会话总线正在运行
- 需要安装 `dbus-monitor` 工具（通常包含在 dbus 包中）
- 按 Ctrl+C 可以停止程序

## 文件格式

通知以文本格式保存，每条记录包含：

```
[2026-01-02 14:30:45] 应用名称
通知标题
通知内容
==================================================
```

## 示例

监听通知并保存到指定目录，每12小时清理一次：

```bash
notify-reg --path ~/Documents/notifications.txt --cleanup-interval 43200
```

测试捕获带超时参数的通知：

```bash
notify-reg -p ~/test.txt &
notify-send -t 1500 "测试" "测试内容"
```
