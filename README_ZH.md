# PP-OCR-RS

<div align="center">

![Rust](https://img.shields.io/badge/rust-1.84+-orange.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/pp-ocr-rs.svg)](https://crates.io/crates/pp-ocr-rs)

**高性能 · Rust 实现的 OCR 引擎**

基于 Paddle OCR onnx 模型的图片文字识别服务，兼容OpenAI 的 chat/completions 接口。

[功能特性](#功能特性) • [快速开始](#快速开始) • [CLI 工具](#cli-工具) • [API 服务](#api-服务) • [性能展示](#性能展示)

**Languages:** [English](README.md) | [简体中文](README_ZH.md)

</div>

---

## ✨ 功能特性

- 🚀 **极致性能** - Rust 原生实现，内存安全，零拷贝优化
- 🎯 **高精度识别** - 支持 Paddle OCR v5 最新模型
- 🔧 **灵活配置** - 丰富的参数配置，适应不同场景需求
- 📦 **开箱即用** - 简单的 CLI 工具，无需编程基础
- 🌐 **API 服务** - 内置 HTTP API 服务器，兼容 OpenAI 接口格式
- 🔄 **并发处理** - 多线程支持，高效处理批量请求
- 📊 **详细输出** - 支持 JSON/文本格式，包含置信度信息

---

## 🚀 快速开始

### 安装

```bash
# 克隆项目
git clone https://github.com/go-restream/pp-ocr-rs.git
cd pp-ocr-rs

# 构建 CLI 工具
cargo build --release

# 构建 API 服务（启用 server 功能）
cargo build --release --features server
```

### 基本使用

```bash
# 识别单张图片
./target/release/ocr image.png

# 批量识别目录中的图片
./target/release/ocr /path/to/images/

# 启动 API 服务
./target/release/ocr serve -c config.yaml
```

---

## 🛠️ CLI 工具

### 命令行参数

```bash
OCR 引擎 - 使用 Paddle OCR 进行图片文字识别

用法: ocr [OPTIONS] <INPUT>

参数:
  <INPUT>    图片文件或包含图片的目录路径

选项:
  -c, --config <FILE>             YAML 配置文件路径
  -f, --format <FORMAT>           输出格式 [text|json] [默认: text]
  -o, --output <FILE>             输出文件路径
      --append                    追加模式（输出到文件时）
  -r, --recursive                 递归处理子目录
  -q, --quiet                     静默模式
  -v, --verbose                   详细模式
      --pretty-json               JSON 美化输出
      --include-confidence        包含置信度信息
      --include-processing-time   包含处理时间信息
  -h, --help                      打印帮助信息
```

### 配置文件示例

创建 `config.yaml`:

```yaml
# 模型路径
det_model_path: "./models/ch_PP-OCRv5_mobile_det.onnx"
cls_model_path: "./models/ch_ppocr_mobile_v2.0_cls_infer.onnx"
rec_model_path: "./models/ch_PP-OCRv5_rec_mobile_infer.onnx"

# 功能开关
use_angle_cls: false
use_direction_cls: false

# 检测参数
detection:
  box_limit: 50
  box_thresh: 0.5
  min_box_size: 0.3
  unclip_ratio: 1.6

# 输出设置
output:
  include_confidence: true
  pretty_json: true
  include_processing_time: true
```

---

## 🌐 API 服务

### 启动服务

```bash
# 使用默认配置（监听 0.0.0.0:8080）
./target/release/ocr serve

# 使用配置文件
./target/release/ocr serve --config config.yaml

# 自定义绑定地址和线程数
./target/release/ocr serve --bind 127.0.0.1:9000 --threads 8
```

### API 接口

#### 健康检查

```bash
curl http://localhost:8080/v1/health
```

#### 获取模型列表

```bash
curl http://localhost:8080/v1/models
```

#### OCR 识别（OpenAI 兼容格式）

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ch_pp_ocr_v5_mobile",
    "messages": [{
      "role": "user",
      "content": [{
        "type": "image_url",
        "image_url": {
          "url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB..."
        }
      }]
    }]
  }'
```

### Python 客户端示例

```python
import base64
import requests

# 读取图片
with open('image.png', 'rb') as f:
    image_data = base64.b64encode(f.read()).decode('utf-8')

# 发送 OCR 请求
response = requests.post(
    'http://localhost:8080/v1/chat/completions',
    json={
        'model': 'ch_pp_ocr_v5_mobile',
        'messages': [{
            'role': 'user',
            'content': [{
                'type': 'image_url',
                'image_url': {
                    'url': f'data:image/png;base64,{image_data}'
                }
            }]
        }]
    }
)

# 解析结果
result = response.json()
text = result['choices'][0]['message']['content'][0]['text']
print(f"识别结果: {text}")
```

---

## 📊 性能展示

### 识别效果

| 测试图片 | 识别结果 | 置信度 |
|---------|---------|--------|
| ![test1](docs/img/test_1.png) | 使用 Rust 通过 ONNX Runtime 调用 Paddle OCR 模型进行图片文字识别。 | 95.27% |
| ![test2](docs/img/test_2.png) | 母婴用品连锁 | 99.71% |

### 性能指标

- **处理速度**: 移动端模型 < 100ms/张（CPU）
- **内存占用**: < 200MB（单实例）
- **并发能力**: 支持多线程并发处理
- **准确率**: 中文场景 > 95%

---

## 🎯 模型支持

| 模型名称 | 类型 | 特点 | 适用场景 |
|---------|------|------|---------|
| ch_pp_ocr_v5_mobile | 移动端 | 速度快，体积小 | 实时处理、移动设备 |
| ch_pp_ocr_v5_server | 服务端 | 精度高，效果好 | 批量处理、高精度需求 |

---

## 📝 开发指南

### 环境要求

- Rust 1.84+
- ONNX Runtime 2.0+

### 本地开发

```bash
# 安装依赖
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example ocr_demo
```

### API 使用示例

```rust
use pp_ocr_rs::{OcrLite, OcrError};

fn main() -> Result<(), OcrError> {
    let mut ocr = OcrLite::new();

    // 初始化模型
    ocr.init_models(
        "./models/det.onnx",
        "./models/cls.onnx",
        "./models/rec.onnx",
        2, // 线程数
    )?;

    // 识别图片
    let result = ocr.detect_from_path(
        "test.png",
        50,    // box_limit
        1024,  // max_box_size
        0.5,   // box_thresh
        0.3,   // min_box_size
        1.6,   // unclip_ratio
        false, // use_angle_cls
        false, // use_direction_cls
    )?;

    // 输出结果
    for block in result.text_blocks {
        println!("文本: {} (置信度: {:.2}%)",
                block.text,
                block.text_score * 100.0);
    }

    Ok(())
}
```

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📄 许可证

本项目采用 [Apache License 2.0](LICENSE) 许可证。

---

## 🙏 致谢

- [Rapid OCR](https://github.com/RapidAI/RapidOCR) - 优秀的 OCR 模型框架
- [Paddle-ocr-rs](https://github.com/mg-chao/paddle-ocr-rs) - Paddle OCR Rust模型库


---

<div align="center">

**如果这个项目对你有帮助，请给个 ⭐️ 支持一下！**

[GitHub](https://github.com/go-restream/pp-ocr-rs) • [文档](docs/) • [示例](examples/)

</div>