# システムアーキテクチャドキュメント

## 概要

本ドキュメントは、Rust APIサーバーのシステムアーキテクチャを詳細に記述します。このシステムは、Actix-webフレームワークをベースとしたREST APIサーバーで、LDAP統合認証、JWT認証、OpenTelemetry可観測性を提供します。

## システム構成図

```mermaid
graph TB
    Client[クライアント<br/>Web/Mobile App]
    
    subgraph "API Server"
        Gateway[HTTP Gateway<br/>Actix-web]
        
        subgraph "Middleware Layer"
            CORS[CORS Middleware]
            Tracing[Tracing Middleware]
            Logger[Logger Middleware]
            ReqData[Request Data Creator]
            JWTAuth[JWT Auth Middleware]
        end
        
        subgraph "Presentation Layer"
            AuthAPI[Auth API<br/>/login]
            UserAPI[User API<br/>/api/users]
            CustomerAPI[Customer API<br/>/api/customers]
            SwaggerUI[Swagger UI<br/>/swagger-ui]
        end
        
        subgraph "Application Layer"
            UserUC[User Use Cases]
            CustomerUC[Customer Use Cases]
            Validation[Validation Logic]
        end
        
        subgraph "Domain Layer"
            UserModel[User Model]
            CustomerModel[Customer Model]
            Traits[Common Traits]
        end
        
        subgraph "Infrastructure Layer"
            DieselORM[Diesel ORM]
            ConnPool[Connection Pool<br/>r2d2]
            Metrics[Metrics Collection]
        end
    end
    
    subgraph "External Systems"
        LDAP[LDAP Server<br/>Active Directory]
        DB[(PostgreSQL<br/>Database)]
        OTEL[OpenTelemetry<br/>Collector]
    end
    
    Client -->|HTTP Request| Gateway
    Gateway --> CORS
    CORS --> Tracing
    Tracing --> Logger
    Logger --> ReqData
    ReqData --> JWTAuth
    
    JWTAuth --> AuthAPI
    JWTAuth --> UserAPI
    JWTAuth --> CustomerAPI
    JWTAuth --> SwaggerUI
    
    AuthAPI --> UserUC
    UserAPI --> UserUC
    CustomerAPI --> CustomerUC
    
    UserUC --> Validation
    CustomerUC --> Validation
    
    UserUC --> UserModel
    CustomerUC --> CustomerModel
    
    UserModel --> Traits
    CustomerModel --> Traits
    
    UserModel --> DieselORM
    CustomerModel --> DieselORM
    
    DieselORM --> ConnPool
    ConnPool --> DB
    
    AuthAPI --> LDAP
    
    Tracing --> Metrics
    Metrics --> OTEL
    
    style Gateway fill:#e1f5ff
    style DB fill:#ffe1e1
    style LDAP fill:#ffe1e1
    style OTEL fill:#e1ffe1
```

## レイヤー構造

本システムは、クリーンアーキテクチャの原則に基づいた4層構造を採用しています。

### 1. Presentation Layer (プレゼンテーション層)

**責務**: HTTPリクエストの受付、レスポンスの生成、ルーティング

**コンポーネント**:
- **Actix-web Handlers**: HTTPエンドポイントの実装
- **Middleware**: リクエスト前処理、認証、トレーシング
- **Swagger UI**: API仕様の可視化

**主要ファイル**:
```
src/
├── main.rs              # サーバー起動、ミドルウェア設定
├── middleware.rs        # JWT認証、トレーシング、リクエストデータ作成
├── services/
│   ├── auth.rs         # 認証エンドポイント (/login)
│   ├── api.rs          # 認証必須エンドポイント設定
│   ├── api/users.rs    # ユーザーAPI
│   └── api/customers.rs # 顧客カテゴリAPI
└── swagger.rs          # OpenAPI定義
```

**データフロー**:
1. HTTPリクエスト受信
2. ミドルウェアチェーン実行 (CORS → Tracing → Logger → ReqData → JWTAuth)
3. ハンドラー関数呼び出し
4. Application Layerへの委譲
5. HTTPレスポンス生成

### 2. Application Layer (アプリケーション層)

**責務**: ビジネスロジックの実装、ユースケースの調整、バリデーション

**コンポーネント**:
- **Use Cases**: ビジネスロジックの実装
- **Validation**: 入力データの検証

**主要ファイル**:
```
src/
├── models/
│   ├── users/usecases.rs      # ユーザー関連ビジネスロジック
│   └── customers/usecases.rs  # 顧客カテゴリ関連ビジネスロジック
└── traits.rs                  # バリデーショントレイト
```

**主要機能**:
- ユーザー検索・登録
- 顧客カテゴリのCRUD操作
- データバリデーション
- トランザクション管理

### 3. Domain Layer (ドメイン層)

**責務**: ドメインモデルの定義、ビジネスルールの表現

**コンポーネント**:
- **Models**: エンティティ定義
- **Traits**: 共通インターフェース

**主要ファイル**:
```
src/
├── models/
│   ├── users.rs      # Userモデル定義
│   └── customers.rs  # CustomerCategoryモデル定義
├── traits.rs         # IntoValidatorトレイト
└── schema.rs         # Diesel自動生成スキーマ
```

**ドメインモデル**:
- **User**: システムユーザー (LDAP連携)
- **CustomerCategory**: 顧客分類

### 4. Infrastructure Layer (インフラストラクチャ層)

**責務**: 外部システムとの連携、データ永続化、可観測性

**コンポーネント**:
- **Diesel ORM**: データベースアクセス
- **Connection Pool**: コネクション管理
- **LDAP Client**: LDAP認証
- **OpenTelemetry**: トレーシング・メトリクス

**主要ファイル**:
```
src/
├── lib.rs           # コネクションプール、テレメトリ初期化
├── config.rs        # 環境変数設定
├── errors.rs        # エラー型定義
├── metrics.rs       # メトリクス収集
└── schema.rs        # データベーススキーマ
```

## モジュール構成

### ディレクトリ構造

```
rust-api/
├── src/
│   ├── main.rs                      # エントリーポイント
│   ├── lib.rs                       # ライブラリルート
│   ├── config.rs                    # 設定管理
│   ├── errors.rs                    # エラー定義
│   ├── middleware.rs                # ミドルウェア
│   ├── traits.rs                    # 共通トレイト
│   ├── schema.rs                    # DBスキーマ
│   ├── swagger.rs                   # OpenAPI定義
│   ├── metrics.rs                   # メトリクス収集
│   ├── models/                      # ドメインモデル
│   │   ├── mod.rs
│   │   ├── users.rs
│   │   ├── users/usecases.rs
│   │   ├── customers.rs
│   │   └── customers/usecases.rs
│   ├── services/                    # APIエンドポイント
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── api.rs
│   │   ├── api/users.rs
│   │   └── api/customers.rs
│   └── bin/                         # CLIツール
│       └── generate_openapi.rs
├── migrations/                      # DBマイグレーション
├── tests/                           # 統合テスト
├── Cargo.toml                       # 依存関係定義
├── diesel.toml                      # Diesel設定
├── Dockerfile                       # コンテナイメージ
└── docker-compose.test.yml          # テスト環境
```

### モジュール依存関係

```mermaid
graph TD
    Main[main.rs] --> Lib[lib.rs]
    Main --> Services[services/*]
    Main --> Swagger[swagger.rs]
    Main --> Middleware[middleware.rs]
    
    Services --> Models[models/*]
    Services --> Errors[errors.rs]
    Services --> Config[config.rs]
    
    Middleware --> Models
    Middleware --> Config
    Middleware --> Metrics[metrics.rs]
    
    Models --> Schema[schema.rs]
    Models --> Traits[traits.rs]
    Models --> Errors
    
    Lib --> Config
    Lib --> Schema
    
    Metrics --> Config
    
    style Main fill:#e1f5ff
    style Lib fill:#e1f5ff
    style Services fill:#ffe1e1
    style Models fill:#e1ffe1
    style Middleware fill:#fff4e1
```

## データフロー

### 1. 認証フロー (LDAP + JWT)

```mermaid
sequenceDiagram
    participant C as Client
    participant G as Gateway
    participant A as Auth Handler
    participant L as LDAP
    participant U as User UseCase
    participant D as Database
    
    C->>G: POST /login<br/>{username, password}
    G->>A: Route to auth handler
    A->>L: Simple Bind認証
    L-->>A: 認証成功
    A->>L: ユーザー情報検索
    L-->>A: employeeNumber, 氏名, email等
    A->>L: Partnerグループチェック
    L-->>A: グループメンバーシップ
    A->>U: search_user(username)
    U->>D: SELECT * FROM users WHERE login_id = ?
    D-->>U: User or Empty
    alt ユーザーが存在しない
        A->>U: insert_new_user(...)
        U->>D: INSERT INTO users
        D-->>U: New User
    end
    A->>A: JWT生成 (有効期限7日)
    A-->>G: 200 OK + Authorization Header
    G-->>C: Response with JWT
```

### 2. 認証済みAPIリクエストフロー

```mermaid
sequenceDiagram
    participant C as Client
    participant M as Middleware Chain
    participant H as Handler
    participant UC as Use Case
    participant D as Database
    participant O as OpenTelemetry
    
    C->>M: GET /api/users/<br/>Authorization: Bearer {token}
    
    Note over M: TracingMiddleware
    M->>O: Create span (http_request)
    M->>O: Record http.method, http.target
    
    Note over M: ReqDataCreatorMiddleware
    M->>M: Decode JWT token
    M->>D: Fetch user from DB
    M->>M: Set ApiRequestData
    
    Note over M: JWTAuthMiddleware
    M->>M: Validate JWT token
    M->>O: Record auth.token_valid
    
    M->>H: Forward request
    H->>UC: Call use case
    UC->>O: Create span (db operation)
    UC->>D: Execute query
    D-->>UC: Query result
    UC-->>H: Return data
    H-->>M: JSON response
    M->>O: Record http.status_code
    M->>O: Record metrics
    M-->>C: 200 OK + JSON body
```

### 3. エラーハンドリングフロー

```mermaid
graph TD
    A[Handler] -->|web::block| B[Use Case]
    B -->|validate| C{Validation}
    C -->|OK| D[Database Operation]
    C -->|Error| E[ValidationError]
    D -->|Success| F[Return Result]
    D -->|Error| G[InternalServerError]
    E --> H[ServiceError]
    G --> H
    H --> I[ResponseError trait]
    I -->|400| J[Bad Request Response]
    I -->|500| K[Internal Server Error Response]
    F --> L[200 OK Response]
    
    style E fill:#ffe1e1
    style G fill:#ffe1e1
    style H fill:#ffe1e1
```

## 技術スタック

### コア技術

| カテゴリ | 技術 | バージョン | 用途 |
|---------|------|-----------|------|
| 言語 | Rust | Edition 2021 | システム実装言語 |
| Webフレームワーク | Actix-web | 4.x | HTTPサーバー、ルーティング |
| ORM | Diesel | 2.0 | データベースアクセス |
| 認証 | jsonwebtoken | 9.x | JWT生成・検証 |
| LDAP | ldap3 | 0.11 | LDAP認証 |
| API仕様 | utoipa | 3.x | OpenAPI生成 |
| バリデーション | validator | 0.16 | 入力検証 |
| 可観測性 | OpenTelemetry | 0.20+ | トレーシング・メトリクス |

### 依存クレート (主要)

```toml
[dependencies]
# Web Framework
actix-web = "4"
actix-cors = "0.7"
actix-web-httpauth = "0.8"

# Database
diesel = { version = "2.0", features = ["postgres", "r2d2"] }
diesel_migrations = "2.0"

# Authentication
jsonwebtoken = "9"
ldap3 = "0.11"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# API Documentation
utoipa = { version = "3", features = ["actix_extras"] }
utoipa-swagger-ui = { version = "3", features = ["actix-web"] }

# Validation
validator = { version = "0.16", features = ["derive"] }

# Observability
opentelemetry = "0.20"
opentelemetry-otlp = "0.13"
tracing = "0.1"
tracing-opentelemetry = "0.21"
tracing-subscriber = "0.3"

# Utilities
dotenvy = "0.15"
envy = "0.4"
chrono = "0.4"
uuid = { version = "1.0", features = ["v4"] }
```

## 可観測性アーキテクチャ

### OpenTelemetry統合

```mermaid
graph LR
    subgraph "API Server"
        A[Tracing Middleware]
        B[Use Cases]
        C[Metrics Collection]
    end
    
    subgraph "OpenTelemetry SDK"
        D[Tracer Provider]
        E[Meter Provider]
        F[OTLP Exporter]
    end
    
    subgraph "Backend"
        G[Jaeger/Tempo]
        H[Prometheus]
    end
    
    A -->|Spans| D
    B -->|Spans| D
    C -->|Metrics| E
    D --> F
    E --> F
    F -->|gRPC| G
    F -->|gRPC| H
    
    style A fill:#e1f5ff
    style B fill:#e1f5ff
    style C fill:#e1f5ff
    style F fill:#ffe1e1
```

### 収集データ

#### トレース (Traces)
- HTTPリクエスト (method, path, status_code, user_agent)
- データベースクエリ (operation, duration)
- LDAP認証 (bind, search)
- JWT検証

#### メトリクス (Metrics)
- `http_requests_total`: リクエスト総数
- `http_request_duration_seconds`: リクエスト処理時間
- `http_requests_in_flight`: 同時実行リクエスト数
- `db_queries_total`: クエリ総数
- `db_query_duration_seconds`: クエリ実行時間
- `auth_attempts_total`: 認証試行回数
- `jwt_validations_total`: JWT検証回数

#### ログ (Logs)
- 構造化ログ (tracing)
- エラーログ (error, warn)
- デバッグログ (debug, trace)

## セキュリティアーキテクチャ

### 認証・認可

```mermaid
graph TD
    A[Client Request] --> B{Path Check}
    B -->|/login| C[No Auth Required]
    B -->|/api/*| D[JWT Auth Required]
    B -->|/swagger-ui| C
    
    D --> E[Extract Bearer Token]
    E --> F{Validate JWT}
    F -->|Invalid| G[401 Unauthorized]
    F -->|Valid| H[Decode Claims]
    H --> I[Fetch User from DB]
    I --> J[Set Request Context]
    J --> K[Forward to Handler]
    
    C --> K
    
    style G fill:#ffe1e1
    style K fill:#e1ffe1
```

### セキュリティ対策

| 対策 | 実装状況 | 詳細 |
|------|---------|------|
| SQLインジェクション | ✅ 実装済み | Diesel ORMのprepared statement |
| 認証 | ✅ 実装済み | LDAP + JWT |
| 認可 | ✅ 実装済み | JWTミドルウェア |
| CORS | ✅ 実装済み | actix-cors |
| パスワード保護 | ✅ 実装済み | LDAPのみ、DB保存なし |
| CSRF | ⚠️ 未実装 | 今後の改善項目 |
| レート制限 | ⚠️ 未実装 | 今後の改善項目 |

## データベースアーキテクチャ

### コネクションプール

```mermaid
graph LR
    A[Actix-web Workers] -->|Get Connection| B[r2d2 Pool]
    B -->|Manage| C[Connection 1]
    B -->|Manage| D[Connection 2]
    B -->|Manage| E[Connection N]
    C --> F[(PostgreSQL)]
    D --> F
    E --> F
    
    style B fill:#e1f5ff
    style F fill:#ffe1e1
```

**設定**:
- プールサイズ: デフォルト (CPU数に基づく)
- タイムアウト: 30秒
- 接続テスト: 取得時に実行

### マイグレーション管理

```mermaid
graph TD
    A[diesel_migrations] --> B[Embedded Migrations]
    B --> C[Migration 1: diesel_initial_setup]
    B --> D[Migration 2: create_users]
    B --> E[Migration 3: create_customer_categories]
    
    F[Test Execution] --> G[Revert All]
    G --> H[Run Pending]
    H --> I[Test with Clean DB]
    
    style B fill:#e1f5ff
    style I fill:#e1ffe1
```

## デプロイメントアーキテクチャ

### Docker構成

```mermaid
graph TD
    subgraph "Docker Compose"
        A[API Server Container]
        B[PostgreSQL Container]
        C[LDAP Server]
        D[Jaeger Container]
    end
    
    A -->|TCP 5432| B
    A -->|TCP 389| C
    A -->|gRPC 4317| D
    
    E[Client] -->|HTTP 8080| A
    E -->|HTTP 16686| D
    
    style A fill:#e1f5ff
    style B fill:#ffe1e1
    style C fill:#ffe1e1
    style D fill:#e1ffe1
```

### 環境変数

| 変数名 | 必須 | デフォルト | 説明 |
|--------|------|-----------|------|
| DATABASE_URL | ✅ | - | PostgreSQL接続URL |
| TEST_DATABASE_URL | ✅ | - | テスト用DB接続URL |
| JWT_SECRET | ✅ | - | JWTシークレットキー (hex) |
| LDAP_URI | ✅ | - | LDAP接続URI |
| LDAP_FILTER | ✅ | - | LDAPフィルター |
| LDAP_UID_COLUMN | ✅ | - | LDAP UID属性名 |
| LDAP_USER_DN | ✅ | - | LDAPユーザーベースDN |
| CLIENT_HOST | ❌ | http://localhost:3000 | CORS許可オリジン |
| OTEL_ENABLED | ❌ | false | OpenTelemetry有効化 |
| OTEL_ENDPOINT | ❌ | http://localhost:4317 | OTLP endpoint |
| OTEL_SERVICE_NAME | ❌ | rust-api | サービス名 |
| OTEL_SERVICE_VERSION | ❌ | 0.1.0 | サービスバージョン |
| RUST_LOG | ❌ | debug | ログレベル |

## パフォーマンス特性

### 非同期処理

- **Actix-web**: 非同期ランタイム (Tokio)
- **web::block**: ブロッキング処理 (DB操作) を別スレッドで実行
- **Connection Pool**: 並行リクエストの効率的な処理

### ボトルネック

| 箇所 | 影響 | 対策 |
|------|------|------|
| LDAP認証 | ログイン時のレイテンシ | キャッシング検討 |
| データベースクエリ | 一覧取得時 | ページネーション実装 |
| JWT検証 | 全リクエスト | 軽量な処理、最適化済み |

## 拡張性

### 水平スケーリング

```mermaid
graph TD
    A[Load Balancer] --> B[API Server 1]
    A --> C[API Server 2]
    A --> D[API Server N]
    
    B --> E[(PostgreSQL)]
    C --> E
    D --> E
    
    B --> F[LDAP]
    C --> F
    D --> F
    
    style A fill:#e1f5ff
    style E fill:#ffe1e1
    style F fill:#ffe1e1
```

**特性**:
- ステートレス設計 (JWT認証)
- 共有データベース
- セッション不要

### 垂直スケーリング

- CPU: Actix-webワーカー数の増加
- メモリ: コネクションプール拡大
- ディスク: データベースストレージ

## 今後の改善方向

### 短期 (1-3ヶ月)
- CSRF対策の実装
- レート制限の追加
- ページネーションの実装
- エラーハンドリングの改善

### 中期 (3-6ヶ月)
- ロールベースアクセス制御 (RBAC)
- 監査ログ機能
- Redisキャッシング
- GraphQL API

### 長期 (6ヶ月以上)
- マイクロサービス化
- Kubernetes対応
- イベント駆動アーキテクチャ
- gRPC API

## 参考資料

- [Actix-web Documentation](https://actix.rs/)
- [Diesel ORM Guide](https://diesel.rs/)
- [OpenTelemetry Rust](https://github.com/open-telemetry/opentelemetry-rust)
- [utoipa Documentation](https://docs.rs/utoipa/)
