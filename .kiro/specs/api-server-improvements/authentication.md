# 認証フロードキュメント

## 概要

本ドキュメントは、Rust APIサーバーの認証システムの詳細フローを記述します。本システムは、LDAP統合認証とJWTトークンベース認証の2段階認証を採用しています。

## 認証アーキテクチャ

### 認証方式の概要

```mermaid
graph LR
    A[Client] -->|1. Login| B[LDAP Authentication]
    B -->|2. Success| C[JWT Generation]
    C -->|3. Return Token| A
    A -->|4. API Request + JWT| D[JWT Validation]
    D -->|5. Success| E[API Access]
    
    style B fill:#e1f5ff
    style C fill:#e1ffe1
    style D fill:#fff4e1
    style E fill:#e1ffe1
```

### 認証フロー全体像

1. **初回認証 (LDAP)**: ユーザー名とパスワードでLDAP認証
2. **トークン発行**: 認証成功時にJWTトークンを発行
3. **トークン保存**: クライアントがトークンを保存
4. **API認証 (JWT)**: 以降のAPIリクエストでJWTトークンを使用
5. **トークン更新**: トークン期限切れ時に再ログイン

## LDAP認証フロー

### 詳細シーケンス図

```mermaid
sequenceDiagram
    participant C as Client
    participant API as API Server
    participant LDAP as LDAP Server
    participant DB as PostgreSQL
    
    Note over C,DB: Phase 1: ログインリクエスト
    C->>API: POST /login<br/>{username: "user001", password: "***"}
    
    Note over API: Extract credentials
    API->>API: Parse LoginInfo
    
    Note over C,DB: Phase 2: LDAP接続
    API->>LDAP: Connect to LDAP URI
    LDAP-->>API: Connection established
    
    Note over C,DB: Phase 3: LDAP Bind認証
    API->>LDAP: Simple Bind<br/>DN: uid=user001,ou=users,dc=example,dc=com<br/>Password: ***
    
    alt 認証成功
        LDAP-->>API: Bind Success
        Note over API: Tracing: auth.ldap_bind = "success"<br/>Metrics: auth_attempts_total{success=true}
        
        Note over C,DB: Phase 4: グループチェック (Partner除外)
        API->>LDAP: Search Groups<br/>Filter: (&(cn=Partner)(objectCategory=CN=Group*))
        LDAP-->>API: Group members list
        
        alt ユーザーがPartnerグループに所属
            API-->>C: 403 Forbidden
            Note over API: Login denied: Partner group member
        else ユーザーがPartnerグループに非所属
            Note over C,DB: Phase 5: ユーザー情報取得
            API->>LDAP: Search User<br/>Filter: (&(uid=user001)(objectClass=person))<br/>Attributes: employeeNumber, sn, givenName, mail, gecos
            LDAP-->>API: User attributes<br/>{employeeNumber: 12345, sn: "山田", givenName: "太郎", ...}
            
            Note over C,DB: Phase 6: データベース確認・登録
            API->>DB: SELECT * FROM users WHERE login_id = 'user001'
            
            alt ユーザーが存在する
                DB-->>API: Existing user record
                Note over API: User found: user_id = 123
            else ユーザーが存在しない
                API->>DB: INSERT INTO users<br/>(login_id, employee_number, first_name, last_name, email, gecos)<br/>VALUES ('user001', 12345, '太郎', '山田', 'taro@example.com', ...)
                DB-->>API: New user record (id: 456)
                Note over API: New user created: user_id = 456
            end
            
            Note over C,DB: Phase 7: JWT生成
            API->>API: Generate JWT<br/>Claims: {id: 123, username: "user001", exp: 7 days}
            API->>API: Sign with HS256 + secret
            
            Note over C,DB: Phase 8: レスポンス返却
            API-->>C: 200 OK<br/>Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
            Note over C: Store JWT token
        end
        
    else 認証失敗
        LDAP-->>API: Bind Failed
        Note over API: Tracing: auth.ldap_bind = "failed"<br/>Metrics: auth_attempts_total{success=false}
        API-->>C: 401 Unauthorized
    end
```

### LDAP認証の詳細

#### 1. LDAP接続

**設定項目**:
- `LDAP_URI`: LDAP接続URI (例: `ldap://ldap.example.com:389`)
- `LDAP_USER_DN`: ユーザーベースDN (例: `ou=users,dc=example,dc=com`)
- `LDAP_UID_COLUMN`: UID属性名 (例: `uid`)

**実装**:
```rust
let (conn, mut ldap) = LdapConnAsync::new(&config.ldap_uri).await?;
ldap3::drive!(conn);
```

#### 2. Simple Bind認証

**DN構築**:
```
{LDAP_UID_COLUMN}={username}, {LDAP_USER_DN}
例: uid=user001,ou=users,dc=example,dc=com
```

**実装**:
```rust
let dn = format!("{}={}, {}", 
    &config.ldap_uid_column, 
    &info.username, 
    &config.ldap_user_dn
);
let result = ldap.simple_bind(&dn, &info.password).await?;
```

**トレーシング**:
- Span: `ldap_bind`
- Attributes: `auth.ldap_bind = "success" | "failed"`

#### 3. Partnerグループチェック

**目的**: 特定グループ (Partner) のユーザーのログインを拒否

**フィルター**:
```ldap
(&(cn=Partner)(objectCategory=CN=Group*))
```

**実装**:
```rust
let guard_filter = "(&(cn=Partner)(objectCategory=CN=Group*))";
let guard_result = ldap.search(
    &config.ldap_user_dn, 
    ldap3::Scope::OneLevel, 
    &guard_filter, 
    vec!["member"]
).await?;

// メンバーリストにユーザーが含まれているかチェック
if members.iter().any(|member| member.contains(&info.username)) {
    return Ok(HttpResponse::Forbidden().finish())
}
```

#### 4. ユーザー情報検索

**フィルター**:
```ldap
(&({LDAP_UID_COLUMN}={username}){LDAP_FILTER})
例: (&(uid=user001)(objectClass=person))
```

**取得属性**:
- `employeeNumber`: 社員番号
- `sn`: 姓 (Surname)
- `givenName`: 名
- `mail`: メールアドレス
- `gecos`: GECOS情報

**実装**:
```rust
let search_filter = format!(
    "(&({}={}){})",&config.ldap_uid_column, 
    &info.username, 
    &config.ldap_filter
);
let search_entry = ldap.search(
    &config.ldap_user_dn, 
    ldap3::Scope::OneLevel, 
    &search_filter, 
    vec!["employeeNumber", "sn", "givenName", "mail", "gecos"]
).await?;
```

**トレーシング**:
- Span: `ldap_user_search`
- Attributes: `auth.user_search = "success"`

#### 5. データベース登録

**ユーザー検索**:
```sql
SELECT * FROM users WHERE login_id = ?
```

**新規ユーザー登録**:
```sql
INSERT INTO users (login_id, employee_number, first_name, last_name, email, gecos)
VALUES (?, ?, ?, ?, ?, ?)
RETURNING *
```

**実装**:
```rust
// 既存ユーザー検索
let users = search_user(&mut conn, &info.username)?;

// 存在しない場合は新規登録
if users.is_empty() {
    let user = insert_new_user(
        &mut conn,
        info.username.to_string(),
        employee_number,
        first_name,
        last_name,
        email,
        gecos
    )?;
}
```

## JWT認証フロー

### JWT生成

#### トークン構造

**Header**:
```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

**Payload (Claims)**:
```json
{
  "id": 123,
  "username": "user001",
  "exp": 1704067200
}
```

**Signature**:
```
HMACSHA256(
  base64UrlEncode(header) + "." + base64UrlEncode(payload),
  secret
)
```

#### 実装詳細

**Claims定義**:
```rust
#[derive(Serialize, Deserialize)]
pub struct UserClaims {
    pub id: i32,           // ユーザーID
    pub username: String,  // ログインID
    pub exp: i64,          // 有効期限 (Unix timestamp)
}
```

**トークン生成**:
```rust
let claims = UserClaims {
    id: user.id,
    username: user.login_id,
    exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp()
};

let secret = config.jwt_secret
    .split(" ")
    .map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap())
    .collect::<Vec<u8>>();

let token = encode(
    &Header::default(), 
    &claims, 
    &EncodingKey::from_secret(&secret)
)?;
```

**有効期限**: 7日間

**レスポンス**:
```http
HTTP/1.1 200 OK
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6MTIzLCJ1c2VybmFtZSI6InVzZXIwMDEiLCJleHAiOjE3MDQwNjcyMDB9.signature
```

### JWT検証フロー

#### 詳細シーケンス図

```mermaid
sequenceDiagram
    participant C as Client
    participant MW1 as TracingMiddleware
    participant MW2 as ReqDataCreatorMiddleware
    participant MW3 as JWTAuthMiddleware
    participant H as Handler
    participant DB as Database
    
    Note over C,DB: Phase 1: リクエスト受信
    C->>MW1: GET /api/users<br/>Authorization: Bearer {token}
    
    Note over MW1: Create tracing span
    MW1->>MW2: Forward request
    
    Note over C,DB: Phase 2: リクエストデータ作成
    MW2->>MW2: Extract Bearer token from header
    MW2->>MW2: Decode JWT (no validation yet)
    
    alt Token is valid format
        MW2->>MW2: Extract username from claims
        MW2->>DB: SELECT * FROM users WHERE login_id = ?
        DB-->>MW2: User record
        MW2->>MW2: Set ApiRequestData with user info
        Note over MW2: User context available for handlers
    else Token is invalid format
        Note over MW2: Skip user lookup, continue without user context
    end
    
    MW2->>MW3: Forward request with extensions
    
    Note over C,DB: Phase 3: JWT認証
    MW3->>MW3: Extract Bearer token
    MW3->>MW3: Validate JWT signature
    MW3->>MW3: Check expiration
    
    alt Token is valid
        Note over MW3: Tracing: auth.token_valid = true<br/>Metrics: jwt_validations_total{valid=true}
        MW3->>H: Forward to handler
        H->>H: Process business logic
        H-->>MW3: Response
        MW3-->>MW2: Response
        MW2-->>MW1: Response
        MW1-->>C: 200 OK + JSON body
    else Token is invalid or expired
        Note over MW3: Tracing: auth.token_valid = false<br/>Metrics: jwt_validations_total{valid=false}
        MW3-->>C: 401 Unauthorized
    end
```

#### ミドルウェアチェーン

```mermaid
graph TD
    A[Request] --> B[TracingMiddleware]
    B --> C[Logger]
    C --> D[ReqDataCreatorMiddleware]
    D --> E[JWTAuthMiddleware]
    E --> F[Handler]
    
    B -.->|Create span| G[OpenTelemetry]
    D -.->|Decode JWT| H[Extract user]
    E -.->|Validate JWT| I[Check signature & exp]
    
    style B fill:#e1f5ff
    style D fill:#fff4e1
    style E fill:#ffe1e1
    style F fill:#e1ffe1
```

#### 1. TracingMiddleware

**責務**: HTTPリクエストのトレーシング

**処理**:
- リクエストIDの生成 (UUID v4)
- トレーシングスパンの作成
- HTTPメトリクスの記録

**トレース属性**:
- `http.method`: HTTPメソッド
- `http.target`: リクエストパス
- `http.user_agent`: User-Agentヘッダー
- `http.status_code`: レスポンスステータスコード
- `request_id`: 一意なリクエストID

#### 2. ReqDataCreatorMiddleware

**責務**: リクエストコンテキストの作成

**処理フロー**:
```rust
1. Authorizationヘッダーからトークンを抽出
2. JWTをデコード (検証なし)
3. Claimsからusernameを取得
4. データベースからユーザー情報を取得
5. ApiRequestDataをリクエストextensionsに設定
```

**実装**:
```rust
pub struct ApiRequestData {
    user: Option<User>
}

// リクエストextensionsに設定
req.extensions_mut().insert(req_data);

// ハンドラーで取得
let req_data = req.extensions().get::<ApiRequestData>();
```

**注意点**:
- このミドルウェアはトークンの検証を行わない
- ユーザー情報の取得のみを目的とする
- 認証失敗時もリクエストを継続

#### 3. JWTAuthMiddleware (actix-web-httpauth)

**責務**: JWT認証の実行

**処理フロー**:
```rust
1. Authorizationヘッダーからトークンを抽出
2. Bearer認証スキームの確認
3. JWTの署名検証 (HS256)
4. 有効期限のチェック
5. 検証成功 → リクエスト継続
6. 検証失敗 → 401 Unauthorized
```

**実装**:
```rust
#[tracing::instrument(skip(token), fields(auth.token_valid = tracing::field::Empty))]
fn validate_token(token: &str) -> Result<bool, Error> {
    let secret = config::get_config().unwrap().jwt_secret;
    let secret = secret.split(" ")
        .map(|hex_str| u8::from_str_radix(hex_str, 16).unwrap())
        .collect::<Vec<u8>>();

    match decode::<UserClaims>(
        &token, 
        &DecodingKey::from_secret(&secret), 
        &Validation::new(Algorithm::HS256)
    ) {
        Ok(_claims) => {
            tracing::Span::current().record("auth.token_valid", true);
            AuthMetrics::record_jwt_validation(true);
            Ok(true)
        }
        Err(err) => {
            tracing::Span::current().record("auth.token_valid", false);
            AuthMetrics::record_jwt_validation(false);
            Err(err).map_err(error::ErrorInternalServerError)
        }
    }
}
```

**トレーシング**:
- Span: `validate_token`
- Attributes: `auth.token_valid = true | false`

**メトリクス**:
- `jwt_validations_total{valid="true"}`: 検証成功数
- `jwt_validations_total{valid="false"}`: 検証失敗数

### JWT検証エラー

| エラー | 原因 | HTTPステータス |
|--------|------|---------------|
| InvalidToken | トークン形式が不正 | 401 |
| InvalidSignature | 署名が一致しない | 401 |
| ExpiredSignature | トークンが期限切れ | 401 |
| ImmatureSignature | トークンがまだ有効でない | 401 |
| InvalidIssuer | 発行者が一致しない | 401 |
| InvalidAudience | 対象者が一致しない | 401 |

## エンドポイント別認証要件

### 認証不要エンドポイント

| Method | Path | 説明 |
|--------|------|------|
| POST | /login | LDAP認証 + JWT発行 |
| GET | /swagger-ui/ | Swagger UI |
| GET | /api-doc/openapi.json | OpenAPI仕様 |

### 認証必須エンドポイント

| Method | Path | 説明 |
|--------|------|------|
| GET | /api/users/ | ユーザー一覧取得 |
| GET | /api/customers/categories | カテゴリ一覧取得 |
| POST | /api/customers/categories | カテゴリ作成 |
| GET | /api/customers/categories/{id} | カテゴリ詳細取得 |
| PUT | /api/customers/categories/{id}/edit | カテゴリ更新 |
| DELETE | /api/customers/categories/{id}/delete | カテゴリ削除 |

## セキュリティ考慮事項

### 実装済みセキュリティ対策

#### 1. パスワード保護
- パスワードはデータベースに保存しない
- LDAP認証のみで使用
- メモリ上でも最小限の保持

#### 2. トークンセキュリティ
- HS256アルゴリズムによる署名
- シークレットキーは環境変数で管理
- 有効期限: 7日間

#### 3. HTTPS推奨
- 本番環境ではHTTPS必須
- トークンの盗聴防止

#### 4. CORS設定
- 許可オリジンの明示的な設定
- 認証ヘッダーの許可

### 今後の改善項目

#### 1. トークンリフレッシュ
**現状**: トークン期限切れ時は再ログイン必要

**改善案**:
- リフレッシュトークンの導入
- アクセストークン: 短期 (1時間)
- リフレッシュトークン: 長期 (30日)

#### 2. トークン無効化
**現状**: トークン無効化の仕組みなし

**改善案**:
- Redisでトークンブラックリスト管理
- ログアウト時にトークンを無効化

#### 3. レート制限
**現状**: ログインエンドポイントにレート制限なし

**改善案**:
- actix-limitationクレートの導入
- IPアドレスベースのレート制限
- 例: 5回/分、100回/時間

#### 4. 多要素認証 (MFA)
**現状**: パスワード認証のみ

**改善案**:
- TOTP (Time-based One-Time Password)
- SMS認証
- メール認証

#### 5. 監査ログ
**現状**: 基本的なログのみ

**改善案**:
- 認証試行の詳細ログ
- 失敗理由の記録
- IPアドレス、User-Agentの記録

## トラブルシューティング

### よくある問題

#### 1. LDAP接続エラー

**症状**:
```
Failed to connect to LDAP server: Connection refused
```

**原因**:
- LDAP_URIが不正
- LDAPサーバーが起動していない
- ネットワーク接続の問題

**対処法**:
```bash
# LDAP接続テスト
ldapsearch -x -H ldap://ldap.example.com:389 -b "dc=example,dc=com"

# 環境変数確認
echo $LDAP_URI
```

#### 2. JWT検証エラー

**症状**:
```
401 Unauthorized
```

**原因**:
- トークンが期限切れ
- JWT_SECRETが不正
- トークン形式が不正

**対処法**:
```bash
# トークンのデコード (検証なし)
echo "eyJhbGc..." | base64 -d

# JWT_SECRET確認
echo $JWT_SECRET

# 再ログイン
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"user001","password":"***"}'
```

#### 3. Partnerグループチェックエラー

**症状**:
```
403 Forbidden
```

**原因**:
- ユーザーがPartnerグループに所属

**対処法**:
- LDAPでグループメンバーシップを確認
- 別のユーザーでログイン

#### 4. ユーザー情報取得エラー

**症状**:
```
Failed to search user in database
```

**原因**:
- データベース接続エラー
- ユーザーテーブルが存在しない

**対処法**:
```bash
# マイグレーション実行
diesel migration run

# データベース接続確認
psql $DATABASE_URL -c "SELECT * FROM users LIMIT 1;"
```

## パフォーマンス最適化

### 現在のパフォーマンス特性

| 処理 | 平均レイテンシ | 備考 |
|------|--------------|------|
| LDAP認証 | 100-500ms | ネットワーク依存 |
| JWT生成 | 1-5ms | CPU依存 |
| JWT検証 | 1-5ms | CPU依存 |
| DB検索 | 5-20ms | インデックス有効 |

### 最適化案

#### 1. LDAP接続プール
**現状**: リクエストごとに接続

**改善案**:
```rust
// LDAP接続プールの導入
pub type LdapPool = r2d2::Pool<LdapConnectionManager>;
```

#### 2. JWT検証キャッシュ
**現状**: リクエストごとに検証

**改善案**:
```rust
// Redisでトークン検証結果をキャッシュ
// TTL: トークン有効期限まで
```

#### 3. ユーザー情報キャッシュ
**現状**: リクエストごとにDB検索

**改善案**:
```rust
// Redisでユーザー情報をキャッシュ
// TTL: 5分
```

## テスト戦略

### 認証テストの種類

#### 1. LDAP認証テスト

**テストケース**:
- 正常系: 有効な認証情報でログイン成功
- 異常系: 無効なパスワードでログイン失敗
- 異常系: 存在しないユーザーでログイン失敗
- 異常系: Partnerグループユーザーでログイン拒否

**実装例**:
```rust
#[actix_web::test]
async fn test_login_success() {
    // LDAPモックサーバーを使用
    let app = test::init_service(App::new().service(login)).await;
    
    let req = test::TestRequest::post()
        .uri("/login")
        .set_json(LoginInfo {
            username: "user001".into(),
            password: "password".into()
        })
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(resp.headers().contains_key("authorization"));
}
```

#### 2. JWT検証テスト

**テストケース**:
- 正常系: 有効なトークンで認証成功
- 異常系: 期限切れトークンで認証失敗
- 異常系: 不正な署名で認証失敗
- 異常系: トークンなしで認証失敗

**実装例**:
```rust
#[actix_web::test]
async fn test_jwt_validation() {
    let token = generate_test_token();
    
    let req = test::TestRequest::get()
        .uri("/api/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}
```

## 参考資料

- [RFC 7519: JSON Web Token (JWT)](https://tools.ietf.org/html/rfc7519)
- [RFC 4511: Lightweight Directory Access Protocol (LDAP)](https://tools.ietf.org/html/rfc4511)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [jsonwebtoken Documentation](https://docs.rs/jsonwebtoken/)
- [ldap3 Documentation](https://docs.rs/ldap3/)
