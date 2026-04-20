# lit-auth

[中文版](./README_CN.md)

A minimalist authentication service designed to work with OpenResty (resty-lua) for unified authorization.

## Overview

`lit-auth` is a microservice focused on "Register, Login, and Logout" functionalities. It doesn't intercept business requests directly; instead, it works alongside OpenResty, which handles global authorization efficiently via Lua scripts.

## How It Works

1.  **Unified Authorization (OpenResty)**:
    Use `access_by_lua_block` in OpenResty to intercept specific URL prefixes (e.g., `/api`). The Lua script reads the `token` from the user's Cookie and attempts to load the session from Redis.
    - If the session doesn't exist, it returns `HTTP 401 Unauthorized`.
    - Core Logic: `session_exists = redis.get("auth:" .. cookie_token)`.

2.  **Routing Rules**:
    Typically, the `/auth` prefix is proxied to the `lit-auth` service.
    - Login Page: `/auth/login`
    - Register Page: `/auth/register`
    - Login API: `/auth/api/login`
    - Register API: `/auth/api/register`

3.  **Frontend Interaction**:
    When frontend JavaScript accesses `/api/xxx` and receives a `401` error, it redirects the user to `/auth/login`.

4.  **State Management**:
    - **Login Success**: `lit-auth` sets a random string as the `token` in the Cookie and stores it in Redis with the key `auth:<token>`.
    - **Logout**: Performs the reverse operation by deleting the key from Redis and clearing the Cookie.

## Core Architecture

-   **Backend**: Rust (Axum + SQLx)
-   **Database**: SQLite (User info storage)
-   **Session Storage**: Redis (Login state storage)
-   **Gateway**: OpenResty / Nginx + Lua

## Configuration

The project uses `config.toml` for configuration:

```toml
listen_addr = "127.0.0.1:3000"
database_url = "sqlite:auth.db"
redis_url = "redis://127.0.0.1:6379"
allow_register = true
```

## OpenResty Configuration Example

Here is a reference Nginx configuration for authorization with `lit-auth`:

```nginx
server {
    listen 80;
    server_name example.com;

    # 1. Proxy /auth to the lit-auth application
    location /auth/ {
        proxy_pass http://127.0.0.1:3000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # 2. Authorize business APIs
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

            -- Get token from cookie
            local token = ngx.var.cookie_token
            if not token then
                ngx.status = 401
                ngx.exit(401)
            end

            -- Check if session exists in redis (Note the 'auth:' prefix)
            local res, err = red:get("auth:" .. token)
            if not res or res == ngx.null then
                ngx.status = 401
                ngx.exit(401)
            end
        }

        # Proxy to actual business service after successful authorization
        proxy_pass http://backend_api_service;
    }
}
```

## Development & Running

1.  **Prepare Database**:
    The project uses SQLite. You can use `sqlx-cli` for migrations:
    ```bash
    export DATABASE_URL="sqlite:auth.db"
    sqlx database setup
    ```

2.  **Build and Run**:
    ```bash
    cargo run -- config.toml
    ```
