# APIエンドポイント一覧

## 概要

本ドキュメントは、Rust APIサーバーが提供する全エンドポイントの詳細仕様を記述します。

## ベースURL

```
http://localhost:8080
```

## 認証

### 認証方式

本APIは、JWT (JSON Web Token) ベースの認証を使用します。

**認証ヘッダー**:
```http
Authorization: Bearer {JWT_TOKEN}
```

### 認証フロー

1. `/login` エンドポイントでLDAP認証を実行
2. 認証成功時、レスポンスヘッダーでJWTトークンを取得
3. 以降のAPIリクエストで、Authorizationヘッダーにトークンを含める

## エンドポイント一覧

### 認証エンドポイント

| Method | Path | 認証 | 説明 |
|--------|------|------|------|
| POST | /login | 不要 | LDAP認証 + JWT発行 |

### ユーザーエンドポイント

| Method | Path | 認証 | 説明 |
|--------|------|------|------|
| GET | /api/users/ | 必要 | ユーザー一覧取得 |

### 顧客カテゴリエンドポイント

| Method | Path | 認証 | 説明 |
|--------|------|------|------|
| GET | /api/customers/categories | 必要 | カテゴリ一覧取得 |
| POST | /api/customers/categories | 必要 | カテゴリ作成 |
| GET | /api/customers/categories/{id} | 必要 | カテゴリ詳細取得 |
| PUT | /api/customers/categories/{id}/edit | 必要 | カテゴリ更新 |
| DELETE | /api/customers/categories/{id}/delete | 必要 | カテゴリ削除 |

### ドキュメントエンドポイント

| Method | Path | 認証 | 説明 |
|--------|------|------|------|
| GET | /swagger-ui/ | 不要 | Swagger UI |
| GET | /api-doc/openapi.json | 不要 | OpenAPI仕様 |

## エンドポイント詳細


---

## 1. 認証API

### POST /login

LDAP認証を実行し、JWTトークンを発行します。

#### リクエスト

**Headers**:
```http
Content-Type: application/json
```

**Body**:
```json
{
  "username": "user001",
  "password": "password123"
}
```

**Body Schema**:
| フィールド | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| username | string | ✅ | LDAPログインID |
| password | string | ✅ | パスワード |

#### レスポンス

**成功 (200 OK)**:
```http
HTTP/1.1 200 OK
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6MTIzLCJ1c2VybmFtZSI6InVzZXIwMDEiLCJleHAiOjE3MDQwNjcyMDB9.signature
```

**失敗 (401 Unauthorized)**:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (403 Forbidden)** - Partnerグループユーザー:
```http
HTTP/1.1 403 Forbidden
```

**失敗 (500 Internal Server Error)**:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### 処理フロー

```mermaid
sequenceDiagram
    participant C as Client
    participant API as API Server
    participant LDAP as LDAP Server
    participant DB as Database
    
    C->>API: POST /login
    API->>LDAP: Simple Bind認証
    alt 認証成功
        LDAP-->>API: Success
        API->>LDAP: ユーザー情報取得
        LDAP-->>API: User attributes
        API->>LDAP: Partnerグループチェック
        alt Partnerグループメンバー
            API-->>C: 403 Forbidden
        else 非メンバー
            API->>DB: ユーザー検索/登録
            API->>API: JWT生成
            API-->>C: 200 OK + JWT
        end
    else 認証失敗
        LDAP-->>API: Failed
        API-->>C: 401 Unauthorized
    end
```

#### cURLサンプル

```bash
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user001",
    "password": "password123"
  }'
```

#### レスポンス例

```http
HTTP/1.1 200 OK
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```


---

## 2. ユーザーAPI

### GET /api/users/

システムに登録されているユーザー一覧を取得します。

#### リクエスト

**Headers**:
```http
Authorization: Bearer {JWT_TOKEN}
```

**Query Parameters**: なし

#### レスポンス

**成功 (200 OK)**:
```json
[
  {
    "id": 1,
    "login_id": "user001",
    "employee_number": 12345,
    "first_name": "太郎",
    "last_name": "山田",
    "email": "taro.yamada@example.com",
    "gecos": "Taro Yamada"
  },
  {
    "id": 2,
    "login_id": "user002",
    "employee_number": 12346,
    "first_name": "花子",
    "last_name": "佐藤",
    "email": "hanako.sato@example.com",
    "gecos": "Hanako Sato"
  }
]
```

**Response Schema**:
| フィールド | 型 | NULL許容 | 説明 |
|-----------|-----|---------|------|
| id | integer | ❌ | ユーザーID |
| login_id | string | ❌ | ログインID |
| employee_number | integer | ✅ | 社員番号 |
| first_name | string | ✅ | 名 |
| last_name | string | ✅ | 姓 |
| email | string | ✅ | メールアドレス |
| gecos | string | ✅ | GECOS情報 |

**失敗 (401 Unauthorized)** - トークン無効:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (500 Internal Server Error)**:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### cURLサンプル

```bash
# トークン取得
TOKEN=$(curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user001","password":"password123"}' \
  -s -D - -o /dev/null | grep -i authorization | cut -d' ' -f2-)

# ユーザー一覧取得
curl -X GET http://localhost:8080/api/users/ \
  -H "Authorization: Bearer $TOKEN"
```

#### 注意事項

- 現在、ページネーション機能は実装されていません
- 全ユーザーが一度に返却されます
- 大量のユーザーが存在する場合、パフォーマンスに影響する可能性があります


---

## 3. 顧客カテゴリAPI

### GET /api/customers/categories

顧客カテゴリの一覧を取得します。

#### リクエスト

**Headers**:
```http
Authorization: Bearer {JWT_TOKEN}
```

**Query Parameters**: なし

#### レスポンス

**成功 (200 OK)**:
```json
[
  {
    "id": 1,
    "name": "VIP顧客"
  },
  {
    "id": 2,
    "name": "一般顧客"
  },
  {
    "id": 3,
    "name": "新規顧客"
  }
]
```

**Response Schema**:
| フィールド | 型 | NULL許容 | 説明 |
|-----------|-----|---------|------|
| id | integer | ❌ | カテゴリID |
| name | string | ❌ | カテゴリ名 (最大255文字) |

**失敗 (401 Unauthorized)**:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (500 Internal Server Error)**:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### cURLサンプル

```bash
curl -X GET http://localhost:8080/api/customers/categories \
  -H "Authorization: Bearer $TOKEN"
```

#### 注意事項

- カテゴリはID昇順でソートされます
- ページネーション機能は実装されていません

---

### POST /api/customers/categories

新しい顧客カテゴリを作成します。

#### リクエスト

**Headers**:
```http
Authorization: Bearer {JWT_TOKEN}
Content-Type: application/json
```

**Body**:
```json
{
  "name": "新規カテゴリ"
}
```

**Body Schema**:
| フィールド | 型 | 必須 | 制約 | 説明 |
|-----------|-----|------|------|------|
| name | string | ✅ | 最大255文字 | カテゴリ名 |

#### レスポンス

**成功 (200 OK)**:
```json
{
  "id": 4,
  "name": "新規カテゴリ"
}
```

**失敗 (400 Bad Request)** - バリデーションエラー:
```json
{
  "name": [
    {
      "code": "length",
      "message": "顧客分類は255文字以下で入力してください",
      "params": {
        "value": "...",
        "max": 255
      }
    }
  ]
}
```

**失敗 (401 Unauthorized)**:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (500 Internal Server Error)**:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### cURLサンプル

```bash
curl -X POST http://localhost:8080/api/customers/categories \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "新規カテゴリ"
  }'
```

#### バリデーションルール

- `name`: 必須、最大255文字


---

### GET /api/customers/categories/{id}

指定されたIDの顧客カテゴリの詳細を取得します。

#### リクエスト

**Headers**:
```http
Authorization: Bearer {JWT_TOKEN}
```

**Path Parameters**:
| パラメータ | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| id | integer | ✅ | カテゴリID |

#### レスポンス

**成功 (200 OK)**:
```json
{
  "id": 1,
  "name": "VIP顧客"
}
```

**失敗 (401 Unauthorized)**:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (500 Internal Server Error)** - カテゴリが存在しない場合も含む:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### cURLサンプル

```bash
curl -X GET http://localhost:8080/api/customers/categories/1 \
  -H "Authorization: Bearer $TOKEN"
```

---

### PUT /api/customers/categories/{id}/edit

指定されたIDの顧客カテゴリを更新します。

#### リクエスト

**Headers**:
```http
Authorization: Bearer {JWT_TOKEN}
Content-Type: application/json
```

**Path Parameters**:
| パラメータ | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| id | integer | ✅ | カテゴリID |

**Body**:
```json
{
  "name": "更新されたカテゴリ名"
}
```

**Body Schema**:
| フィールド | 型 | 必須 | 制約 | 説明 |
|-----------|-----|------|------|------|
| name | string | ✅ | 最大255文字 | カテゴリ名 |

#### レスポンス

**成功 (200 OK)**:
```json
{
  "id": 1,
  "name": "更新されたカテゴリ名"
}
```

**失敗 (400 Bad Request)** - バリデーションエラー:
```json
{
  "name": [
    {
      "code": "length",
      "message": "顧客分類は255文字以下で入力してください",
      "params": {
        "value": "...",
        "max": 255
      }
    }
  ]
}
```

**失敗 (401 Unauthorized)**:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (500 Internal Server Error)** - カテゴリが存在しない場合も含む:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### cURLサンプル

```bash
curl -X PUT http://localhost:8080/api/customers/categories/1/edit \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "更新されたカテゴリ名"
  }'
```

#### バリデーションルール

- `name`: 必須、最大255文字


---

### DELETE /api/customers/categories/{id}/delete

指定されたIDの顧客カテゴリを削除します。

#### リクエスト

**Headers**:
```http
Authorization: Bearer {JWT_TOKEN}
```

**Path Parameters**:
| パラメータ | 型 | 必須 | 説明 |
|-----------|-----|------|------|
| id | integer | ✅ | カテゴリID |

#### レスポンス

**成功 (200 OK)**:
```json
{
  "id": 1,
  "name": "削除されたカテゴリ"
}
```

**失敗 (401 Unauthorized)**:
```http
HTTP/1.1 401 Unauthorized
```

**失敗 (500 Internal Server Error)** - カテゴリが存在しない場合も含む:
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

"Internal Server Error, Please try later"
```

#### cURLサンプル

```bash
curl -X DELETE http://localhost:8080/api/customers/categories/1/delete \
  -H "Authorization: Bearer $TOKEN"
```

#### 注意事項

- 削除されたカテゴリの情報がレスポンスとして返却されます
- 現在、カテゴリに関連する顧客データは存在しないため、外部キー制約はありません
- 将来的に顧客テーブルが追加された場合、外部キー制約により削除が制限される可能性があります


---

## 4. ドキュメントAPI

### GET /swagger-ui/

Swagger UIを表示します。APIの仕様を視覚的に確認し、テストすることができます。

#### リクエスト

**Headers**: なし (認証不要)

#### レスポンス

**成功 (200 OK)**:
```http
HTTP/1.1 200 OK
Content-Type: text/html

<!DOCTYPE html>
<html>
...Swagger UI HTML...
</html>
```

#### ブラウザアクセス

```
http://localhost:8080/swagger-ui/
```

#### 機能

- 全エンドポイントの一覧表示
- リクエスト/レスポンスのスキーマ表示
- 「Try it out」機能でAPIテスト実行
- JWT認証のテスト (Authorizeボタン)

---

### GET /api-doc/openapi.json

OpenAPI 3.0形式のAPI仕様を取得します。

#### リクエスト

**Headers**: なし (認証不要)

#### レスポンス

**成功 (200 OK)**:
```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "Rust API",
    "version": "0.1.0"
  },
  "paths": {
    "/login": {
      "post": {
        "tags": ["auth"],
        "summary": "Login",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "$ref": "#/components/schemas/LoginInfo"
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Login User",
            "headers": {
              "authorization": {
                "schema": {
                  "type": "string"
                }
              }
            }
          }
        }
      }
    }
  },
  "components": {
    "schemas": {
      "LoginInfo": {
        "type": "object",
        "properties": {
          "username": {
            "type": "string"
          },
          "password": {
            "type": "string"
          }
        }
      }
    },
    "securitySchemes": {
      "BearerAuth": {
        "type": "http",
        "scheme": "bearer",
        "bearerFormat": "JWT"
      }
    }
  }
}
```

#### cURLサンプル

```bash
curl -X GET http://localhost:8080/api-doc/openapi.json
```

#### 用途

- API仕様の自動生成
- コード生成ツールへの入力
- ドキュメント生成
- API契約テスト


---

## エラーレスポンス

### エラーコード一覧

| HTTPステータス | 説明 | 発生条件 |
|---------------|------|---------|
| 400 Bad Request | バリデーションエラー | 入力データが不正 |
| 401 Unauthorized | 認証エラー | トークンが無効または期限切れ |
| 403 Forbidden | アクセス拒否 | Partnerグループユーザー |
| 500 Internal Server Error | サーバーエラー | データベースエラー、LDAP接続エラー等 |

### エラーレスポンス形式

#### バリデーションエラー (400)

```json
{
  "field_name": [
    {
      "code": "validation_code",
      "message": "エラーメッセージ",
      "params": {
        "key": "value"
      }
    }
  ]
}
```

**例**:
```json
{
  "name": [
    {
      "code": "length",
      "message": "顧客分類は255文字以下で入力してください",
      "params": {
        "value": "非常に長い文字列...",
        "max": 255
      }
    }
  ]
}
```

#### 認証エラー (401)

```http
HTTP/1.1 401 Unauthorized
```

レスポンスボディなし

#### サーバーエラー (500)

```json
"Internal Server Error, Please try later"
```

---

## CORS設定

### 許可オリジン

デフォルト: `http://localhost:3000`

環境変数 `CLIENT_HOST` で変更可能:
```bash
export CLIENT_HOST=http://example.com:3000
```

### 許可メソッド

- GET
- POST
- PUT
- DELETE

### 許可ヘッダー

- Content-Type
- Authorization

### 公開ヘッダー

- Authorization (JWTトークン取得用)

---

## レート制限

**現状**: レート制限は実装されていません

**今後の実装予定**:
- ログインエンドポイント: 5回/分、100回/時間
- その他のエンドポイント: 1000回/時間

---

## ページネーション

**現状**: ページネーション機能は実装されていません

**今後の実装予定**:

```http
GET /api/users/?page=1&per_page=20
```

**レスポンス例**:
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```


---

## 使用例

### 完全なワークフロー例

#### 1. ログイン

```bash
# ログインしてトークンを取得
RESPONSE=$(curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user001","password":"password123"}' \
  -i)

# Authorizationヘッダーからトークンを抽出
TOKEN=$(echo "$RESPONSE" | grep -i "^authorization:" | cut -d' ' -f2- | tr -d '\r')

echo "Token: $TOKEN"
```

#### 2. ユーザー一覧取得

```bash
curl -X GET http://localhost:8080/api/users/ \
  -H "Authorization: Bearer $TOKEN"
```

#### 3. カテゴリ作成

```bash
curl -X POST http://localhost:8080/api/customers/categories \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"VIP顧客"}'
```

#### 4. カテゴリ一覧取得

```bash
curl -X GET http://localhost:8080/api/customers/categories \
  -H "Authorization: Bearer $TOKEN"
```

#### 5. カテゴリ更新

```bash
curl -X PUT http://localhost:8080/api/customers/categories/1/edit \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"プレミアム顧客"}'
```

#### 6. カテゴリ削除

```bash
curl -X DELETE http://localhost:8080/api/customers/categories/1/delete \
  -H "Authorization: Bearer $TOKEN"
```

---

## Postmanコレクション例

### 環境変数設定

```json
{
  "name": "Rust API Environment",
  "values": [
    {
      "key": "base_url",
      "value": "http://localhost:8080",
      "enabled": true
    },
    {
      "key": "token",
      "value": "",
      "enabled": true
    }
  ]
}
```

### ログインリクエスト

```json
{
  "name": "Login",
  "request": {
    "method": "POST",
    "header": [
      {
        "key": "Content-Type",
        "value": "application/json"
      }
    ],
    "body": {
      "mode": "raw",
      "raw": "{\n  \"username\": \"user001\",\n  \"password\": \"password123\"\n}"
    },
    "url": {
      "raw": "{{base_url}}/login",
      "host": ["{{base_url}}"],
      "path": ["login"]
    }
  },
  "event": [
    {
      "listen": "test",
      "script": {
        "exec": [
          "const token = pm.response.headers.get('Authorization').replace('Bearer ', '');",
          "pm.environment.set('token', token);"
        ]
      }
    }
  ]
}
```

### 認証付きリクエスト

```json
{
  "name": "Get Users",
  "request": {
    "method": "GET",
    "header": [
      {
        "key": "Authorization",
        "value": "Bearer {{token}}"
      }
    ],
    "url": {
      "raw": "{{base_url}}/api/users/",
      "host": ["{{base_url}}"],
      "path": ["api", "users", ""]
    }
  }
}
```

---

## トラブルシューティング

### よくある問題

#### 1. 401 Unauthorized エラー

**原因**:
- トークンが無効
- トークンが期限切れ (7日間)
- Authorizationヘッダーの形式が不正

**対処法**:
```bash
# トークンの確認
echo $TOKEN

# 再ログイン
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user001","password":"password123"}'
```

#### 2. 400 Bad Request エラー

**原因**:
- バリデーションエラー
- リクエストボディの形式が不正

**対処法**:
```bash
# レスポンスボディを確認
curl -X POST http://localhost:8080/api/customers/categories \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"非常に長い文字列..."}' \
  -v
```

#### 3. CORS エラー

**原因**:
- 許可されていないオリジンからのリクエスト

**対処法**:
```bash
# CLIENT_HOST環境変数を設定
export CLIENT_HOST=http://your-frontend-domain.com
```

#### 4. 500 Internal Server Error

**原因**:
- データベース接続エラー
- LDAP接続エラー
- 予期しないサーバーエラー

**対処法**:
```bash
# サーバーログを確認
docker logs rust-api-container

# データベース接続確認
psql $DATABASE_URL -c "SELECT 1;"

# LDAP接続確認
ldapsearch -x -H $LDAP_URI -b $LDAP_USER_DN
```

---

## パフォーマンス特性

### レスポンスタイム (目安)

| エンドポイント | 平均レスポンスタイム | 備考 |
|--------------|-------------------|------|
| POST /login | 100-500ms | LDAP接続に依存 |
| GET /api/users/ | 10-50ms | データ量に依存 |
| GET /api/customers/categories | 5-20ms | データ量に依存 |
| POST /api/customers/categories | 10-30ms | バリデーション含む |
| PUT /api/customers/categories/{id}/edit | 10-30ms | バリデーション含む |
| DELETE /api/customers/categories/{id}/delete | 5-20ms | - |

### 同時接続数

- Actix-webのデフォルト設定: CPU数 × 2 ワーカー
- コネクションプール: デフォルトサイズ

### スループット

- 単純なGETリクエスト: 1000-5000 req/sec (ハードウェアに依存)
- 認証付きリクエスト: 500-2000 req/sec

---

## セキュリティ考慮事項

### 実装済み

- ✅ JWT認証
- ✅ LDAP統合認証
- ✅ CORS設定
- ✅ SQLインジェクション対策 (Diesel ORM)
- ✅ パスワード非保存 (LDAP認証のみ)

### 今後の実装予定

- ⚠️ CSRF対策
- ⚠️ レート制限
- ⚠️ トークンリフレッシュ
- ⚠️ トークン無効化 (ログアウト)
- ⚠️ 多要素認証 (MFA)

### 推奨事項

1. **HTTPS使用**: 本番環境では必ずHTTPSを使用
2. **トークン管理**: クライアント側でトークンを安全に保存 (HttpOnly Cookie推奨)
3. **環境変数**: シークレットキーは環境変数で管理
4. **ログ監視**: 認証失敗のログを監視
5. **定期的な更新**: 依存クレートを定期的に更新

---

## 参考資料

- [Swagger UI](http://localhost:8080/swagger-ui/)
- [OpenAPI Specification](http://localhost:8080/api-doc/openapi.json)
- [Actix-web Documentation](https://actix.rs/)
- [utoipa Documentation](https://docs.rs/utoipa/)
- [JWT.io](https://jwt.io/)
