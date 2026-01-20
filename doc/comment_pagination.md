# Note Comment 分页 payload 指南

本文档详细说明笔记评论接口 (`/api/note/page`) 的分页机制。

## 接口信息

| 项目 | 值 |
|------|-----|
| 端点 | `GET /api/note/page` |
| 描述 | 获取指定笔记的评论列表 |
| 分页方式 | **游标分页** (`cursor`) |
| 每页数量 | 由服务端决定 |

## 请求参数

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `note_id` | string | ✅ | 笔记 ID |
| `cursor` | string | ❌ | 分页游标，首次请求为空 |
| `top_comment_id` | string | ❌ | 置顶评论 ID，通常为空 |
| `xsec_token` | string | ✅ | 笔记的安全令牌 (从 feed/search 结果获取) |
| `image_formats` | string | ❌ | 图片格式: `jpg,webp,avif` |

## 核心分页规则

```
首次请求:  cursor = "" (空字符串)

后续请求:  cursor = 上次返回的 Response.data.cursor

固定字段:  note_id = 保持不变
           xsec_token = 保持不变 (来自笔记详情)
```

## xsec_token 来源

`xsec_token` 是访问笔记相关接口的必需参数，获取途径：

1. **从 Feed 获取**: `/api/feed/homefeed/{category}` 返回的笔记列表中包含
2. **从搜索获取**: `/api/search/notes` 返回的笔记列表中包含
3. **特征**: 与 `note_id` 一一对应，每篇笔记有唯一的 `xsec_token`

## 请求示例

### 第1页 (首次请求)
```
GET /api/note/page?note_id=696b611e000000001a0210de&cursor=&top_comment_id=&image_formats=jpg,webp,avif&xsec_token=ABkT8PPgDgmMj8QDzHMrqEe-bozccvXIirQq865XG_HGA%3D
```

参数拆解：
```
note_id=696b611e000000001a0210de
cursor=                              # 首次为空
top_comment_id=
image_formats=jpg,webp,avif
xsec_token=ABkT8PPgDgmMj8QDzHMrqEe-bozccvXIirQq865XG_HGA%3D
```

### 第2页 (带游标)
```
GET /api/note/page?note_id=696b611e000000001a0210de&cursor=696ef72f000000000800de0b&top_comment_id=&image_formats=jpg,webp,avif&xsec_token=ABkT8PPgDgmMj8QDzHMrqEe-bozccvXIirQq865XG_HGA%3D
```

参数拆解：
```
note_id=696b611e000000001a0210de     # 同上
cursor=696ef72f000000000800de0b      # 来自上次响应
top_comment_id=
image_formats=jpg,webp,avif
xsec_token=ABkT8PPgDgmMj8QDzHMrqEe-bozccvXIirQq865XG_HGA%3D
```

## 响应字段

| 字段 | 说明 |
|------|------|
| `data.cursor` | 下一页游标，用于获取下一页评论 |
| `data.has_more` | 是否有更多评论 (`true`/`false`) |
| `data.comments` | 评论列表 |
| `data.comments[].id` | 评论 ID |
| `data.comments[].content` | 评论内容 |
| `data.comments[].user_info` | 评论者信息 |

## Python 客户端示例

```python
import time
import json
import urllib.request
from urllib.parse import urlencode

BASE_URL = "http://localhost:3005"

def get_note_comments(note_id: str, xsec_token: str, max_pages: int = 5):
    """获取笔记评论分页示例"""
    all_comments = []
    cursor = ""  # 首次为空
    
    for page in range(1, max_pages + 1):
        params = {
            "note_id": note_id,
            "cursor": cursor,
            "top_comment_id": "",
            "image_formats": "jpg,webp,avif",
            "xsec_token": xsec_token
        }
        
        url = f"{BASE_URL}/api/note/page?{urlencode(params)}"
        
        req = urllib.request.Request(url)
        
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            comments = data.get("data", {}).get("comments", [])
            cursor = data.get("data", {}).get("cursor", "")
            has_more = data.get("data", {}).get("has_more", False)
            
            all_comments.extend(comments)
            
            print(f"Page {page}: {len(comments)} comments, cursor: {cursor[:20]}...")
            
            if not has_more or not cursor:
                print("No more comments")
                break
            
            time.sleep(1)  # 间隔请求
        else:
            print(f"Error: {data.get('msg')}")
            break
    
    return all_comments


# 使用示例
if __name__ == "__main__":
    # 1. 首先从 search 或 feed 获取笔记的 note_id 和 xsec_token
    note_id = "696b611e000000001a0210de"
    xsec_token = "ABkT8PPgDgmMj8QDzHMrqEe-bozccvXIirQq865XG_HGA="
    
    # 2. 获取评论
    comments = get_note_comments(note_id, xsec_token, max_pages=3)
    print(f"\nTotal comments: {len(comments)}")
```

## 与其他分页方式对比

| 特性 | Note Comment | Search Notes | Homefeed |
|------|-------------|--------------|----------|
| 分页方式 | 游标 (`cursor`) | 页码 (`page`) | 游标 (`cursor_score` + `note_index`) |
| 复杂度 | ⭐⭐ 中等 | ⭐ 简单 | ⭐⭐⭐ 复杂 |
| 状态依赖 | 仅需 `cursor` | `search_id` 不变 | 需多个参数联动 |
| 结束判断 | `has_more = false` | `has_more = false` | `items` 为空 |
| 额外依赖 | 需要 `xsec_token` | 需要 `search_id` | 无 |

## 注意事项

> [!IMPORTANT]
> **xsec_token 必需**: 每个 `note_id` 对应唯一的 `xsec_token`，必须从 feed 或 search 结果中获取，不能伪造。

> [!TIP]
> **cursor 来源**: 每次请求返回的 `cursor` 用于获取下一页，务必保存。

> [!WARNING]
> **风控提醒**: 请求间隔建议 ≥ 1 秒，避免高频率请求触发封禁。

> [!CAUTION]
> **风险提示**: 评论接口对 `xsec_token` 校验严格，token 失效或不匹配会返回错误。**使用本接口产生的风险由用户自行承担**。
