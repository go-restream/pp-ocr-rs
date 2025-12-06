# PP-OCR-RS 配置指南

> 本指南详细介绍了 PP-OCR-RS 的所有配置选项，帮助您根据实际需求优化服务性能和功能。

## 目录

- [配置文件概述](#配置文件概述)
- [配置文件格式](#配置文件格式)
- [服务配置](#服务配置)
- [模型配置](#模型配置)
- [检测参数配置](#检测参数配置)
- [识别参数配置](#识别参数配置)
- [输出配置](#输出配置)
- [功能开关配置](#功能开关配置)
- [性能优化配置](#性能优化配置)
- [配置示例](#配置示例)
- [故障排除](#故障排除)

## 配置文件概述

PP-OCR-RS 使用 YAML 格式的配置文件来管理各种参数。配置文件允许您自定义服务行为、模型路径、性能参数等。

### 默认配置文件位置

- 配置文件路径：`config.yaml`
- 示例配置文件：`config.yaml.example`

### 配置文件加载优先级

1. 命令行指定的配置文件：`--config <path>`
2. 当前目录下的 `config.yaml`
3. 默认内置配置

## 配置文件格式

配置文件采用 YAML 格式，包含以下主要部分：

```yaml
# 服务配置
server:
  # 服务相关参数

# 模型路径配置
det_model_path: "..."
cls_model_path: "..."
rec_model_path: "..."
dict_path: "..."

# 功能开关
use_angle_cls: false
use_direction_cls: false

# 检测参数
detection:
  # 检测相关参数

# 识别参数
recognition:
  # 识别相关参数

# 输出格式
output:
  # 输出相关参数
```

## 服务配置

### 基础服务参数

```yaml
server:
  # 绑定地址和端口
  bind_addr: "0.0.0.0:8080"

  # 工作线程数
  num_threads: 4

  # 日志级别
  log_level: "info"  # 可选: error, warn, info, debug, trace

  # OCR 引擎池大小（可选）
  ocr_pool_size: null  # null 表示使用 CPU 核心数

  # 启动时是否预热模型
  warmup_on_startup: true
```

#### 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| bind_addr | string | "0.0.0.0:8080" | 服务监听地址 |
| num_threads | int | 4 | 服务处理线程数 |
| log_level | string | "info" | 日志输出级别 |
| ocr_pool_size | int or null | null | OCR 引擎池大小 |
| warmup_on_startup | bool | true | 是否在启动时预热模型 |

### 日志配置

```yaml
# ORT（ONNX Runtime）日志级别
ort_log_level: "warning"  # 可选: verbose, info, warning, error, fatal
```

## 模型配置

### 模型路径配置

```yaml
# 文本检测模型路径
det_model_path: "./models/ch_PP-OCRv5_mobile_det.onnx"

# 文字方向分类模型路径
cls_model_path: "./models/ch_ppocr_mobile_v2.0_cls_infer.onnx"

# 文字识别模型路径
rec_model_path: "./models/ch_PP-OCRv5_rec_mobile_infer.onnx"

# 自定义字典文件路径（可选）
dict_path: "./models/dict.txt"
```

#### 模型选择指南

1. **移动端模型**（推荐用于资源受限环境）
   ```yaml
   det_model_path: "./models/ch_PP-OCRv5_mobile_det.onnx"
   rec_model_path: "./models/ch_PP-OCRv5_rec_mobile_infer.onnx"
   ```

2. **服务端模型**（推荐用于高精度需求）
   ```yaml
   det_model_path: "./models/ch_PP-OCRv5_server_det.onnx"
   rec_model_path: "./models/ch_PP-OCRv5_rec_server_infer.onnx"
   ```

## 检测参数配置

```yaml
detection:
  # 检测框数量限制
  box_limit: 50

  # 文本框置信度阈值 (0.0-1.0)
  box_thresh: 0.5

  # 最小文本框大小阈值
  min_box_size: 3

  # 文本框扩展比例
  unclip_ratio: 1.6

  # 是否使用膨胀操作
  use_dilation: false

  # 是否使用多边形评分
  use_polygon_score: false
```

### 检测参数调优建议

1. **提高检测精度**
   ```yaml
   detection:
     box_thresh: 0.3      # 降低阈值，检测更多候选框
     unclip_ratio: 1.8    # 增加扩展比例
     min_box_size: 2      # 允许更小的文本框
   ```

2. **提高检测速度**
   ```yaml
   detection:
     box_limit: 30        # 限制检测框数量
     box_thresh: 0.7      # 提高阈值，快速过滤
     min_box_size: 5      # 忽略过小的文本框
   ```

## 识别参数配置

```yaml
recognition:
  # 是否使用对数空间计算
  use_log: false
```

## 输出配置

```yaml
output:
  # 是否在输出中包含置信度信息
  include_confidence: false

  # 是否格式化 JSON 输出
  pretty_json: false

  # 是否包含处理时间信息
  include_processing_time: false
```

### 输出格式示例

1. **标准输出**（默认）
   ```json
   {
     "text": "识别的文本内容"
   }
   ```

2. **包含置信度**
   ```json
   {
     "text": "识别的文本内容",
     "confidence": 0.95
   }
   ```

3. **格式化输出**
   ```json
   {
     "text": "识别的文本内容",
     "boxes": [
       {
         "text": "第一行文本",
         "confidence": 0.98,
         "box": [x1, y1, x2, y2, x3, y3, x4, y4]
       }
     ]
   }
   ```

## 功能开关配置

```yaml
# 是否使用文字方向分类
use_angle_cls: false

# 是否使用方向分类
use_direction_cls: false
```

### 功能说明

1. **文字方向分类**（use_angle_cls）
   - 启用后会自动识别图片中的文字方向（0°、90°、180°、270°）
   - 适用于可能包含旋转文字的图片
   - 会增加少量处理时间

2. **方向分类**（use_direction_cls）
   - 区分水平文字和垂直文字
   - 适用于包含垂直排版文字的场景

## 性能优化配置

### 高并发配置

```yaml
server:
  # 根据服务器 CPU 核心数调整
  num_threads: 8
  ocr_pool_size: 8  # 匹配线程数

  # 启用模型预热
  warmup_on_startup: true

  # 设置适当的日志级别
  log_level: "warn"  # 生产环境建议使用 warn 或 error
```

### 低延迟配置

```yaml
# 使用移动端模型
det_model_path: "./models/ch_PP-OCRv5_mobile_det.onnx"
rec_model_path: "./models/ch_PP-OCRv5_rec_mobile_infer.onnx"

# 关闭不必要的功能
use_angle_cls: false
use_direction_cls: false

# 优化检测参数
detection:
  box_limit: 30
  box_thresh: 0.7
```

### 高精度配置

```yaml
# 使用服务端模型
det_model_path: "./models/ch_PP-OCRv5_server_det.onnx"
rec_model_path: "./models/ch_PP-OCRv5_rec_server_infer.onnx"

# 启用所有优化功能
use_angle_cls: true
use_direction_cls: true

# 优化检测参数
detection:
  box_thresh: 0.3
  unclip_ratio: 2.0
  min_box_size: 2

# 输出详细信息
output:
  include_confidence: true
  pretty_json: true
```

## 配置示例

### 开发环境配置

```yaml
# config-dev.yaml
server:
  bind_addr: "127.0.0.1:8080"
  num_threads: 2
  log_level: "debug"
  warmup_on_startup: true

# 使用移动端模型
det_model_path: "./models/ch_PP-OCRv5_mobile_det.onnx"
cls_model_path: "./models/ch_ppocr_mobile_v2.0_cls_infer.onnx"
rec_model_path: "./models/ch_PP-OCRv5_rec_mobile_infer.onnx"

# 开发时启用详细输出
output:
  include_confidence: true
  include_processing_time: true
  pretty_json: true

ort_log_level: "info"
```

### 生产环境配置

```yaml
# config-prod.yaml
server:
  bind_addr: "0.0.0.0:8080"
  num_threads: 8
  log_level: "warn"
  ocr_pool_size: 8
  warmup_on_startup: true

# 使用服务端模型
det_model_path: "/app/models/ch_PP-OCRv5_server_det.onnx"
cls_model_path: "/app/models/ch_ppocr_mobile_v2.0_cls_infer.onnx"
rec_model_path: "/app/models/ch_PP-OCRv5_rec_server_infer.onnx"

# 性能优化
use_angle_cls: false
use_direction_cls: false

detection:
  box_limit: 50
  box_thresh: 0.5
  min_box_size: 3
  unclip_ratio: 1.6

# 精简输出
output:
  include_confidence: false
  pretty_json: false
  include_processing_time: false

ort_log_level: "error"
```

### 高并发场景配置

```yaml
# config-high-concurrency.yaml
server:
  bind_addr: "0.0.0.0:8080"
  num_threads: 16
  log_level: "error"
  ocr_pool_size: 16
  warmup_on_startup: true

# 使用性能最优的模型组合
det_model_path: "./models/ch_PP-OCRv5_server_det.onnx"
rec_model_path: "./models/ch_PP-OCRv5_rec_mobile_infer.onnx"

# 关闭非必要功能
use_angle_cls: false
use_direction_cls: false

# 优化检测参数以提高速度
detection:
  box_limit: 30
  box_thresh: 0.7
  min_box_size: 5
  unclip_ratio: 1.5

# 最小化输出
output:
  include_confidence: false
  pretty_json: false
  include_processing_time: false

ort_log_level: "fatal"
```

## 故障排除

### 常见配置问题

1. **服务无法启动**
   - 检查端口是否被占用：`netstat -tlnp | grep 8080`
   - 验证配置文件语法：`python -c 'import yaml; yaml.safe_load(open("config.yaml"))'`
   - 查看详细错误日志，设置 `log_level: "debug"`

2. **模型加载失败**
   - 确认模型文件路径是否正确
   - 检查模型文件权限
   - 验证模型文件完整性

3. **内存占用过高**
   - 减少 `ocr_pool_size`
   - 使用移动端模型
   - 调整 `num_threads`

4. **处理速度慢**
   - 启用模型预热：`warmup_on_startup: true`
   - 增加 `ocr_pool_size`
   - 优化检测参数

### 性能调优检查清单

- [ ] 使用合适的模型（移动端/服务端）
- [ ] 配置正确的线程数和池大小
- [ ] 关闭不必要的功能
- [ ] 优化检测参数
- [ ] 设置合适的日志级别
- [ ] 启用模型预热

### 配置验证

使用内置的配置验证功能：

```bash
# 验证配置文件
./target/release/ocr --config config.yaml --validate-config

# 查看当前配置
./target/release/ocr --config config.yaml --show-config
```

### 监控和诊断

1. **启用详细日志**
   ```yaml
   server:
     log_level: "debug"
   ort_log_level: "verbose"
   ```

2. **监控性能指标**
   ```yaml
   output:
     include_processing_time: true
   ```

3. **查看统计信息**
   ```bash
   # 发送健康检查请求
   curl http://localhost:8080/v1/health

   # 查看模型列表
   curl http://localhost:8080/v1/models
   ```

## 配置最佳实践

1. **环境分离**
   - 为不同环境使用不同的配置文件
   - 使用环境变量覆盖关键配置

2. **性能优化**
   - 根据硬件配置调整线程池大小
   - 选择合适的模型平衡速度和精度

3. **安全性**
   - 生产环境关闭详细日志
   - 限制服务绑定地址

4. **可维护性**
   - 添加配置注释
   - 使用版本控制管理配置文件

5. **资源管理**
   - 设置合理的资源限制
   - 监控内存和 CPU 使用情况

---

## 更新日志

### v1.0.0
- 初始配置系统
- 支持 YAML 格式配置
- 完整的参数配置选项

## 相关文档

- [API 指南](api.md)
- [部署指南](deployment.md)
- [性能优化](performance.md)

## 联系方式

如有问题或建议，请通过以下方式联系：
- GitHub Issues: [项目地址](https://github.com/go-restream/pp-ocr-rs)
- 邮箱: superspjj@163.com

## 许可证

本项目采用 Apache License 2.0 许可证，详情请见 [LICENSE](../LICENSE) 文件。