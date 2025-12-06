# PP-OCR-RS API 指南

> PP-OCR-RS 是一个基于 Rust 的高性能 OCR（光学字符识别）服务，提供与 OpenAI API 兼容的接口。

## 目录

- [快速开始](#快速开始)
- [API 基础](#api-基础)
- [认证方式](#认证方式)
- [API 端点](#api-端点)
  - [OCR 识别接口](#ocr-识别接口)
  - [健康检查接口](#健康检查接口)
  - [模型列表接口](#模型列表接口)
- [请求参数](#请求参数)
- [响应格式](#响应格式)
- [错误处理](#错误处理)
- [性能优化](#性能优化)
- [SDK 和集成示例](#sdk-和集成示例)
- [常见问题](#常见问题)

## 快速开始

### 1. 启动服务

```bash
# 使用默认配置启动
./target/release/ocr serve

# 使用自定义配置启动
./target/release/ocr serve --config config.yaml
```

### 2. 发送 OCR 请求

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "ch_pp_ocr_v5_mobile",
    "messages": [
      {
        "role": "user",
        "content": [
          {
            "type": "text",
            "text": "ocr"
          },
          {
            "type": "image_url",
            "image_url": {
              "url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
            }
          }
        ]
      }
    ]
  }'
```

## API 基础

### 基础 URL

```
http://localhost:8080
```

### 内容类型

所有 API 请求都需要设置 `Content-Type: application/json` 头部。

### API 版本

当前 API 版本为 `v1`，所有端点都以 `/v1/` 开头。

## 认证方式

目前 API 不需要认证，但建议在生产环境中添加适当的认证机制（如 API Key）。

## API 端点

### OCR 识别接口

**端点：** `POST /v1/chat/completions`

执行 OCR 文字识别，支持多种图片格式。

#### 请求示例

```json
{
  "model": "ch_pp_ocr_v5_mobile",
  "messages": [
    {
      "role": "user",
      "content": [
        {
          "type": "text",
          "text": "ocr"
        },
        {
          "type": "image_url",
          "image_url": {
            "url": "data:image/png;base64,<BASE64_ENCODED_IMAGE>"
          }
        }
      ]
    }
  ]
}
```

#### 支持的模型

- `ch_pp_ocr_v5_mobile` - 移动端优化模型，速度较快，适合资源受限环境
- `ch_pp_ocr_v5_server` - 服务端模型，精度更高，适合对准确性要求高的场景

#### 响应示例

```json
{
  "id": "chatcmpl-12345678-1234-1234-1234-123456789012",
  "object": "chat.completion",
  "created": 1704067200,
  "model": "ch_pp_ocr_v5_mobile",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": [
          {
            "type": "text",
            "text": "识别到的文本内容"
          }
        ]
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 0,
    "completion_tokens": 0,
    "total_tokens": 0
  }
}
```

### 健康检查接口

**端点：** `GET /v1/health`

检查服务是否正常运行。

#### 请求示例

```bash
curl -X GET http://localhost:8080/v1/health
```

#### 响应示例

```json
{
  "status": "ok",
  "timestamp": 1704067200
}
```

### 模型列表接口

**端点：** `GET /v1/models`

获取所有可用的 OCR 模型列表。

#### 请求示例

```bash
curl -X GET http://localhost:8080/v1/models
```

#### 响应示例

```json
{
  "object": "list",
  "data": [
    {
      "id": "ch_pp_ocr_v5_mobile",
      "object": "model",
      "created": 1704067200,
      "owned_by": "pp-ocr"
    },
    {
      "id": "ch_pp_ocr_v5_server",
      "object": "model",
      "created": 1704067200,
      "owned_by": "pp-ocr"
    }
  ]
}
```

## 请求参数

### OCR 识别接口参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| model | string | 是 | 要使用的模型名称 |
| messages | array | 是 | 消息列表，包含图片和文本 |
| messages[].role | string | 是 | 角色，通常为 "user" |
| messages[].content | array | 是 | 内容数组 |
| messages[].content[].type | string | 是 | 内容类型，"text" 或 "image_url" |
| messages[].content[].text | string | 否 | 文本内容，需为 "ocr" |
| messages[].content[].image_url | object | 否 | 图片信息 |
| messages[].content[].image_url.url | string | 是 | Base64 编码的图片 URL |

### 图片格式要求

- 支持的格式：PNG, JPEG, JPG, BMP, TIFF
- 编码方式：Base64
- URL 格式：`data:image/[format];base64,[data]`
- 建议图片大小：不超过 10MB
- 建议分辨率：最小 32x32 像素，最大 4096x4096 像素

## 响应格式

### 成功响应

所有成功响应都包含以下基本字段：

```json
{
  "id": "chatcmpl-<uuid>",
  "object": "chat.completion",
  "created": <unix_timestamp>,
  "model": "<model_name>",
  "choices": [...],
  "usage": {...}
}
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| id | string | 唯一请求标识符 |
| object | string | 对象类型，固定为 "chat.completion" |
| created | integer | 创建时间（Unix 时间戳） |
| model | string | 使用的模型名称 |
| choices | array | 识别结果列表 |
| usage | object | 使用情况统计（当前为占位符） |

## 错误处理

### 错误响应格式

```json
{
  "error": {
    "message": "错误描述",
    "error_type": "error_type",
    "code": "error_code"
  }
}
```

### 常见错误码

| HTTP 状态码 | 错误类型 | 说明 |
|------------|----------|------|
| 400 | invalid_request_error | 请求参数错误 |
| 400 | 无效的模型 | 指定的模型不存在 |
| 400 | 图片解码失败 | Base64 编码错误或格式不支持 |
| 400 | 未找到图片 | 请求中不包含图片数据 |
| 500 | internal_error | 内部服务器错误 |
| 500 | OCR 处理失败 | OCR 引擎处理异常 |

### 错误示例

```json
{
  "error": {
    "message": "Invalid model. Supported models: ch_pp_ocr_v5_mobile, ch_pp_ocr_v5_server",
    "error_type": "invalid_request_error",
    "code": null
  }
}
```

## 性能优化

### 服务端优化

1. **模型预加载**
   ```yaml
   server:
     warmup_on_startup: true  # 启动时预热模型
     ocr_pool_size: 4         # 配置 OCR 引擎池大小
   ```

2. **线程配置**
   ```yaml
   server:
     num_threads: 8  # 根据 CPU 核心数调整
   ```

3. **检测参数调优**
   ```yaml
   detection:
     box_thresh: 0.5      # 调整文本框阈值
     unclip_ratio: 1.6    # 调整文本框扩展比例
   ```

### 客户端优化

1. **批量处理**：对于大量图片，使用异步并发请求

2. **图片预处理**：
   - 适当压缩图片减少传输时间
   - 保持文字清晰度
   - 避免过度放大或旋转

3. **缓存结果**：对相同图片缓存 OCR 结果

### 性能基准

- **移动端模型**：单张图片处理时间 ~200-500ms
- **服务端模型**：单张图片处理时间 ~500-1500ms
- **并发处理**：支持多线程并发，吞吐量取决于 OCR 池大小

## SDK 和集成示例

### Python 示例

```python
import base64
import requests
import json

def ocr_image(image_path, model="ch_pp_ocr_v5_mobile", base_url="http://localhost:8080"):
    """OCR 识别图片文字"""

    # 读取并编码图片
    with open(image_path, "rb") as f:
        image_data = base64.b64encode(f.read()).decode()

    # 构建请求
    url = f"{base_url}/v1/chat/completions"
    payload = {
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "ocr"},
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": f"data:image/png;base64,{image_data}"
                        }
                    }
                ]
            }
        ]
    }

    # 发送请求
    response = requests.post(url, json=payload)
    result = response.json()

    if "error" in result:
        raise Exception(f"OCR Error: {result['error']['message']}")

    return result["choices"][0]["message"]["content"][0]["text"]

# 使用示例
try:
    text = ocr_image("example.png")
    print("识别结果：", text)
except Exception as e:
    print("错误：", e)
```

### JavaScript/Node.js 示例

```javascript
const fs = require('fs');
const axios = require('axios');

async function ocrImage(imagePath, model = 'ch_pp_ocr_v5_mobile', baseUrl = 'http://localhost:8080') {
    try {
        // 读取并编码图片
        const imageBuffer = fs.readFileSync(imagePath);
        const imageBase64 = imageBuffer.toString('base64');

        // 构建请求
        const response = await axios.post(`${baseUrl}/v1/chat/completions`, {
            model: model,
            messages: [
                {
                    role: 'user',
                    content: [
                        { type: 'text', text: 'ocr' },
                        {
                            type: 'image_url',
                            image_url: {
                                url: `data:image/png;base64,${imageBase64}`
                            }
                        }
                    ]
                }
            ]
        });

        return response.data.choices[0].message.content[0].text;
    } catch (error) {
        if (error.response && error.response.data.error) {
            throw new Error(`OCR Error: ${error.response.data.error.message}`);
        }
        throw error;
    }
}

// 使用示例
(async () => {
    try {
        const text = await ocrImage('example.png');
        console.log('识别结果：', text);
    } catch (error) {
        console.error('错误：', error.message);
    }
})();
```

### Go 示例

```go
package main

import (
    "bytes"
    "encoding/base64"
    "encoding/json"
    "fmt"
    "io/ioutil"
    "net/http"
)

type OCRRequest struct {
    Model    string    `json:"model"`
    Messages []Message `json:"messages"`
}

type Message struct {
    Role    string     `json:"role"`
    Content []Content  `json:"content"`
}

type Content struct {
    Type     string    `json:"type"`
    Text     string    `json:"text,omitempty"`
    ImageURL *ImageURL `json:"image_url,omitempty"`
}

type ImageURL struct {
    URL string `json:"url"`
}

type OCRResponse struct {
    ID      string   `json:"id"`
    Object  string   `json:"object"`
    Created int64    `json:"created"`
    Model   string   `json:"model"`
    Choices []Choice `json:"choices"`
}

type Choice struct {
    Index       int     `json:"index"`
    Message     Message `json:"message"`
    FinishReason string `json:"finish_reason"`
}

func ocrImage(imagePath, model, baseURL string) (string, error) {
    // 读取图片
    imageData, err := ioutil.ReadFile(imagePath)
    if err != nil {
        return "", err
    }

    // Base64 编码
    imageBase64 := base64.StdEncoding.EncodeToString(imageData)

    // 构建请求
    req := OCRRequest{
        Model: model,
        Messages: []Message{
            {
                Role: "user",
                Content: []Content{
                    {Type: "text", Text: "ocr"},
                    {
                        Type: "image_url",
                        ImageURL: &ImageURL{
                            URL: fmt.Sprintf("data:image/png;base64,%s", imageBase64),
                        },
                    },
                },
            },
        },
    }

    // 序列化请求
    reqBody, err := json.Marshal(req)
    if err != nil {
        return "", err
    }

    // 发送 HTTP 请求
    resp, err := http.Post(
        fmt.Sprintf("%s/v1/chat/completions", baseURL),
        "application/json",
        bytes.NewBuffer(reqBody),
    )
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()

    // 解析响应
    body, err := ioutil.ReadAll(resp.Body)
    if err != nil {
        return "", err
    }

    var ocrResp OCRResponse
    if err := json.Unmarshal(body, &ocrResp); err != nil {
        return "", err
    }

    if len(ocrResp.Choices) > 0 && len(ocrResp.Choices[0].Message.Content) > 0 {
        return ocrResp.Choices[0].Message.Content[0].Text, nil
    }

    return "", fmt.Errorf("no text found in response")
}

func main() {
    text, err := ocrImage("example.png", "ch_pp_ocr_v5_mobile", "http://localhost:8080")
    if err != nil {
        fmt.Printf("错误: %v\n", err)
        return
    }
    fmt.Printf("识别结果: %s\n", text)
}
```

## 常见问题

### Q: 如何选择合适的模型？

**A:**
- `ch_pp_ocr_v5_mobile`：适合移动端或资源受限环境，处理速度快，占用内存少
- `ch_pp_ocr_v5_server`：适合服务端环境，识别精度更高，但资源消耗更大

### Q: 支持哪些图片格式？

**A:** 支持 PNG、JPEG、JPG、BMP、TIFF 等常见图片格式。建议使用 PNG 或 JPEG 格式以获得最佳效果。

### Q: 图片大小有什么限制？

**A:** 建议图片大小不超过 10MB，分辨率在 32x32 到 4096x4096 像素之间。过大的图片会影响处理速度。

### Q: 如何提高识别准确率？

**A:**
1. 确保图片清晰度，避免模糊
2. 保持适当的对比度
3. 避免文字倾斜或变形
4. 对于服务端模型，可以使用 `include_confidence` 参数查看识别置信度

### Q: API 是否支持批量处理？

**A:** 当前 API 每次请求只能处理一张图片。如需批量处理，请发送多个并发请求。

### Q: 如何配置服务以获得最佳性能？

**A:**
1. 启用模型预热 (`warmup_on_startup: true`)
2. 根据 CPU 核心数配置适当的 OCR 池大小
3. 调整检测参数以适应特定场景

### Q: 服务无法启动怎么办？

**A:**
1. 检查端口是否被占用
2. 确认模型文件路径是否正确
3. 查看日志输出获取详细错误信息

### Q: 如何监控服务状态？

**A:** 使用 `/v1/health` 端点进行健康检查，或查看服务器日志了解运行状态。

---

## 更新日志

### v1.0.0
- 初始版本发布
- 支持 OCR 文字识别
- 提供 OpenAI 兼容 API
- 支持移动端和服务端模型

## 联系方式

如有问题或建议，请通过以下方式联系：
- GitHub Issues: [项目地址](https://github.com/go-restream/pp-ocr-rs)
- 邮箱: superspjj@163.com

## 许可证

本项目采用 Apache License 2.0 许可证，详情请见 [LICENSE](../LICENSE) 文件。