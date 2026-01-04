# 默认安装路径，用户可以通过 make install PREFIX=~/.cargo 来修改
PREFIX ?= $(HOME)/.local
BIN_DIR = $(PREFIX)/bin
SYSTEMD_DIR = $(HOME)/.config/systemd/user

# 二进制文件名
BINARY_NAME = notify-reg
TARGET_RELEASE = target/release/$(BINARY_NAME)

.PHONY: all build install clean

all: build

build:
	@echo "🦀 Building release binary..."
	cargo build --release

install: build
	@echo "📦 Installing binary to $(BIN_DIR)..."
	@mkdir -p $(BIN_DIR)
	@rm -f $(BIN_DIR)/notify-reg
	@cp $(TARGET_RELEASE) $(BIN_DIR)/

	@echo "⚙️  Generating Systemd service..."
	@mkdir -p $(SYSTEMD_DIR)
	@# 使用 sed 替换模板中的占位符，生成最终的 .service 文件
	@sed "s|{{BIN_PATH}}|$(BIN_DIR)/$(BINARY_NAME)|g" notify-reg.service.in > $(SYSTEMD_DIR)/notify-reg.service
	
	@echo "✅ Installation complete!"
	@echo "   Run:"
	@echo "        systemctl --user daemon-reload"
	@echo "        systemctl --user enable --now notify-reg"

clean:
	cargo clean
