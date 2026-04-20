# lit-auth

[English Version](./README.md)


一个配合 OpenResty (resty-lua) 实现统一鉴权的极简身份验证程序。

## 项目简介

`lit-auth` 是一个专注于“注册、登录、退出”功能的微服务。它本身不直接拦截业务请求，而是与 OpenResty 配合，通过 Lua 脚本实现高效的全局鉴权。

## 工作原理

1.  **统一鉴权 (OpenResty)**:
    在 OpenResty 中使用 `access_by_lua_block` 对指定的 URL 前缀（如 `/api`）进行拦截。鉴权脚本从用户 Cookie 中读取 `token`，并根据该 token 从 Redis 中加载 Session。
    - 如果 Session 不存在，返回 `HTTP 401 Unauthorized`。
    - 核心逻辑：`session_exists = redis.get("auth:" .. cookie_token)`。

2.  **路由规则**:
    通常将 `/auth` 前缀代理到 `lit-auth` 程序。
    - 登录页面：`/auth/login`
    - 注册页面：`/auth/register`
    - 登录 API：`/auth/api/login`
    - 注册 API：`/auth/api/register`

3.  **前端交互**:
    前端 JavaScript 访问 `/api/xxx` 时，如果收到 `401` 错误，则重定向到 `/auth/login`。

4.  **状态管理**:
    - **登录成功**: `lit-auth` 会在 Cookie 中设置一个随机字符串作为 `token`，并以 `auth:<token>` 为键存入 Redis。
    - **退出登录**: 执行相反操作，从 Redis 删除键并清除 Cookie。

## 核心架构

-   **Backend**: Rust (Axum + SQLx)
-   **Database**: SQLite (存储用户信息)
-   **Session Storage**: Redis (存储登录态)
-   **Gateway**: OpenResty / Nginx + Lua

## 配置说明

项目使用 `config.toml` 进行配置：

```toml
listen_addr = "127.0.0.1:3000"
database_url = "sqlite:auth.db"
redis_url = "redis://127.0.0.1:6379"
allow_register = true
```

## OpenResty 配置示例

以下是配合 `lit-auth` 进行鉴权的 Nginx 配置参考：

```nginx
server {
    listen 80;
    server_name example.com;

    # 1. 将 /auth 代理到 lit-auth 程序
    location /auth/ {
        proxy_pass http://127.0.0.1:3000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # 2. 对业务 API 进行鉴权
    location /api {
        access_by_lua_block {
            local redis = require "resty.redis"
            local red = redis:new()
            red:set_timeouts(1000, 1000, 1000)

            local ok, err = red:connect("127.0.0.1", 6379)
            if not ok then
                ngx.status = 500
                ngx.say("failed to connect to redis: ", err)
                return ngx.exit(500)
            end

            -- 从 cookie 获取 token
            local token = ngx.var.cookie_token
            if not token then
                ngx.status = 401
                ngx.exit(401)
            end

            -- 检查 redis 中是否存在该 session (注意 prefix 为 auth:)
            local res, err = red:get("auth:" .. token)
            if not res or res == ngx.null then
                ngx.status = 401
                ngx.exit(401)
            end
        }

        # 鉴权通过后反向代理到实际业务服务
        proxy_pass http://backend_api_service;
    }
}
```

## 开发与运行

1.  **准备数据库**:
    项目使用 SQLite，可以使用 `sqlx-cli` 进行迁移：
    ```bash
    export DATABASE_URL="sqlite:auth.db"
    sqlx database setup
    ```

2.  **编译并运行**:
    ```bash
    cargo run -- config.toml
    ```

## 静态页面

`lit-auth` 自带了简单的 HTML 页面（位于 `html/` 目录）：
- `login.html`: 登录页
- `register.html`: 注册页
