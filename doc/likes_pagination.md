# Likes Notification 分页 payload 指南

本文档详细说明赞和收藏通知接口 (`/api/notification/likes`) 的分页机制。

## 接口信息

| 项目 | 值 |
|------|-----|
| 端点 | `GET /api/notification/likes` |
| 描述 | 获取赞和收藏通知列表 |
| 分页方式 | **游标分页** (`cursor`) |
| 每页数量 | **固定 20 条** |

## 请求参数

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `num` | int | ❌ | 每页数量 (固定20，可不传) |
| `cursor` | string | ❌ | 分页游标，首次请求为空，后续使用响应中的 cursor 值 |

## 核心分页规则

```
首次请求:  cursor = "" (空字符串或不传)

后续请求:  cursor = 上次返回的 Response.data.cursor 或 Response.data.strCursor

固定字段:  num = 20 (服务端固定)
```

## 请求示例

### 第1页 (首次请求)
```
GET /api/notification/likes?num=20&cursor=
```

### 第2页
```
GET /api/notification/likes?num=20&cursor=7553158242479239810
```

### 第3页
```
GET /api/notification/likes?num=20&cursor=7530654352792903646
```

### 第4页
```
GET /api/notification/likes?num=20&cursor=7524981894987004255
```

## 响应字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `data.message_list` | array | 通知消息列表 |
| `data.has_more` | bool | 是否有更多数据 (`true`/`false`) |
| `data.cursor` | int64 | 下一页游标 (数值型) |
| `data.strCursor` | string | 下一页游标 (字符串型，推荐使用) |

### 响应示例

```json
{
    "success": true,
    "msg": "",
    "data": {
        "message_list": [
            {
                "id": "...",
                "type": "like",
                "user": {...},
                "note": {...}
            }
        ],
        "has_more": true,
        "cursor": 7553158242479239810,
        "strCursor": "7553158242479239810",
        "result": {
            "code": 0,
            "message": "",
            "success": true
        }
    }
}
```

## Python 客户端示例

```python
import time
import json
import urllib.request
from urllib.parse import urlencode

BASE_URL = "http://localhost:3005"

def get_likes_notification(max_pages: int = 5):
    """获取赞和收藏通知分页示例"""
    all_messages = []
    cursor = ""  # 首次为空
    
    for page in range(1, max_pages + 1):
        params = {
            "num": 20,
            "cursor": cursor
        }
        
        url = f"{BASE_URL}/api/notification/likes?{urlencode(params)}"
        
        req = urllib.request.Request(url)
        
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            messages = data.get("data", {}).get("message_list", [])
            # 优先使用字符串游标
            cursor = data.get("data", {}).get("strCursor") or str(data.get("data", {}).get("cursor", ""))
            has_more = data.get("data", {}).get("has_more", False)
            
            all_messages.extend(messages)
            
            print(f"Page {page}: {len(messages)} messages, cursor: {cursor}")
            
            if not has_more or not cursor:
                print("No more messages")
                break
            
            time.sleep(1)  # 间隔请求
        else:
            print(f"Error: {data.get('msg')}")
            break
    
    return all_messages


# 使用示例
if __name__ == "__main__":
    messages = get_likes_notification(max_pages=3)
    print(f"\nTotal messages: {len(messages)}")
```

## 与其他分页方式对比

| 特性 | Likes Notification | Note Comment | Search Notes |
|------|-------------------|--------------|--------------|
| 分页方式 | 游标 (`cursor`) | 游标 (`cursor`) | 页码 (`page`) |
| 复杂度 | ⭐ 简单 | ⭐⭐ 中等 | ⭐ 简单 |
| 游标类型 | 数值型 (int64) | 字符串 | 无 |
| 状态依赖 | 无额外依赖 | 需要 `xsec_token` | 需要 `search_id` |
| 结束判断 | `has_more = false` | `has_more = false` | `has_more = false` |

## 注意事项

> [!TIP]
> **cursor 来源**: 响应中同时提供 `cursor` (数值) 和 `strCursor` (字符串) 两种格式，推荐使用 `strCursor` 以避免精度问题。

> [!WARNING]
> **风控提醒**: 请求间隔建议 ≥ 1 秒，高频请求可能触发风控。

> [!IMPORTANT]
> **登录要求**: 此接口需要登录态，请确保已完成登录流程。

> [!CAUTION]
> **风险提示**: 通知接口对账号登录状态校验严格，Cookie 失效会返回错误。**使用本接口产生的风险由用户自行承担**。
