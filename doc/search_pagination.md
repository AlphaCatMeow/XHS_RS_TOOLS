# Search Notes 分页 payload 指南

本文档详细说明搜索笔记接口 (`/api/search/notes`) 的分页机制。

## 接口信息

| 项目 | 值 |
|------|-----|
| 端点 | `POST /api/search/notes` |
| 描述 | 获取关键词搜索的笔记列表 |
| 分页方式 | **页码递增** (`page`) |
| 每页数量 | **固定 20 条** |

## 请求参数

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `keyword` | string | ✅ | 搜索关键词 |
| `page` | int | ✅ | **页码 (从1开始，分页唯一需要变更的字段)** |
| `page_size` | int | ❌ | 每页数量 (固定20，无需传递) |
| `sort` | string | ❌ | 排序方式: `general`(综合), `time_descending`(最新) |
| `note_type` | int | ❌ | 笔记类型: 0=综合, 1=图文, 2=视频 |
| `search_id` | string | ✅ | 搜索会话ID (首次生成后保持不变) |
| `ext_flags` | array | ❌ | 扩展筛选标志 (通常为空数组) |
| `geo` | string | ❌ | 地理位置 (通常为空) |
| `image_formats` | array | ❌ | 图片格式: `["jpg", "webp", "avif"]` |

## 核心分页规则

```
分页请求:  仅需递增 page 字段 (1, 2, 3...)

固定字段:  page_size = 20 (服务端固定)
           search_id = 保持不变
           其他字段 = 保持不变
```

## 请求示例

### 第1页
```json
{
    "keyword": "台州招聘",
    "page": 1,
    "page_size": 20,
    "search_id": "2fvihnjxft23yizymtj52",
    "sort": "general",
    "note_type": 0,
    "ext_flags": [],
    "geo": "",
    "image_formats": ["jpg", "webp", "avif"]
}
```

### 第2页
```json
{
    "keyword": "台州招聘",
    "page": 2,
    "page_size": 20,
    "search_id": "2fvihnjxft23yizymtj52",
    "sort": "general",
    "note_type": 0,
    "ext_flags": [],
    "geo": "",
    "image_formats": ["jpg", "webp", "avif"]
}
```

### 第3页及后续
只需将 `page` 递增即可，其他字段保持不变。

## 响应字段

| 字段 | 说明 |
|------|------|
| `data.has_more` | 是否有下一页 (`true`/`false`) |
| `data.items` | 笔记列表 |
| `data.items[].id` | 笔记ID |
| `data.items[].xsec_token` | 访问该笔记详情需要的 token |

## search_id 生成规则

`search_id` 由客户端生成，格式为随机字符串，例如：
- `2fvihnjxft23yizymtj52`
- `demo_sid_1768866964`

建议格式：时间戳 + 随机字符串

## Python 客户端示例

```python
import time
import json
import urllib.request

BASE_URL = "http://localhost:3005"

def search_notes_pagination(keyword: str, max_pages: int = 3):
    """搜索笔记分页示例"""
    search_id = f"search_{int(time.time())}"
    all_notes = []
    
    for page in range(1, max_pages + 1):
        payload = {
            "keyword": keyword,
            "page": page,
            "page_size": 20,
            "search_id": search_id,
            "sort": "general",
            "note_type": 0,
            "ext_flags": [],
            "geo": "",
            "image_formats": ["jpg", "webp", "avif"]
        }
        
        req = urllib.request.Request(
            f"{BASE_URL}/api/search/notes",
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            items = data.get("data", {}).get("items", [])
            has_more = data.get("data", {}).get("has_more", False)
            all_notes.extend(items)
            
            print(f"Page {page}: {len(items)} notes")
            
            if not has_more:
                print("No more pages")
                break
            
            time.sleep(1)  # 间隔请求
        else:
            print(f"Error: {data.get('msg')}")
            break
    
    return all_notes
```

## 与 Homefeed 分页对比

| 特性 | Search Notes | Homefeed |
|------|-------------|----------|
| 分页方式 | 页码递增 (`page`) | 游标 (`cursor_score` + `note_index`) |
| 复杂度 | ⭐ 简单 | ⭐⭐⭐ 复杂 |
| 状态依赖 | `search_id` 保持不变 | 需要上次返回的 `cursor_score` |
| 页码字段 | `page` (1, 2, 3...) | 无，使用 `note_index` 计算 |
| 结束判断 | `has_more = false` | `items` 为空或少于请求数量 |
