# User Search 分页 payload 指南

本文档详细说明用户搜索接口 (`/api/search/usersearch`) 的分页机制。

## 接口信息

| 项目 | 值 |
|------|-----|
| 端点 | `POST /api/search/usersearch` |
| 描述 | 搜索小红书用户 |
| 分页方式 | **页码递增** (`page`) |
| 每页数量 | **固定 15 个** |

## 请求参数

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `keyword` | string | ✅ | 搜索关键词 |
| `page` | int | ✅ | 页码 (从1开始) |
| `page_size` | int | ❌ | 每页数量 (固定15) |
| `search_id` | string | ✅ | 搜索会话ID (与 search/notes 共用) |
| `biz_type` | string | ❌ | 业务类型: `web_search_user` |

## 核心分页规则

```
首次请求:  page = 1, search_id = 自行生成或复用

后续请求:  page = page + 1, search_id = 保持不变

每页固定:  page_size = 15 (服务端固定)
```

## 请求示例

### 第1页
```json
{
    "keyword": "台州招聘",
    "page": 1,
    "page_size": 15,
    "search_id": "2fvihnjxft23yizymtj52",
    "biz_type": "web_search_user"
}
```

### 第2页
```json
{
    "keyword": "台州招聘",
    "page": 2,
    "page_size": 15,
    "search_id": "2fvihnjxft23yizymtj52",
    "biz_type": "web_search_user"
}
```

## 响应字段

| 字段 | 说明 |
|------|------|
| `data.users` | 用户列表 |
| `data.users[].user_id` | 用户ID |
| `data.users[].name` | 用户昵称 |
| `data.users[].red_id` | 红薯号 |
| `data.users[].avatar` | 头像URL |

## Python 客户端示例

```python
import time
import json
import urllib.request

BASE_URL = "http://localhost:3005"

def search_users_pagination(keyword: str, max_pages: int = 3):
    """用户搜索分页示例"""
    search_id = f"search_{int(time.time())}"
    all_users = []
    
    for page in range(1, max_pages + 1):
        payload = {
            "keyword": keyword,
            "page": page,
            "page_size": 15,
            "search_id": search_id,
            "biz_type": "web_search_user"
        }
        
        req = urllib.request.Request(
            f"{BASE_URL}/api/search/usersearch",
            data=json.dumps(payload).encode('utf-8'),
            headers={'Content-Type': 'application/json'}
        )
        
        with urllib.request.urlopen(req, timeout=15) as response:
            data = json.loads(response.read().decode('utf-8'))
        
        if data.get("success"):
            users = data.get("data", {}).get("users", [])
            all_users.extend(users)
            
            print(f"Page {page}: {len(users)} users")
            
            if len(users) < 15:  # 不足15个说明没有更多
                print("No more pages")
                break
            
            time.sleep(1)
        else:
            print(f"Error: {data.get('msg')}")
            break
    
    return all_users
```

## 注意事项

1. `search_id` 可与 `/api/search/notes` 共用，便于在同一搜索会话中切换笔记/用户结果
2. 每页固定返回最多 15 个用户
3. 当返回用户数少于 15 时，说明已无更多结果
