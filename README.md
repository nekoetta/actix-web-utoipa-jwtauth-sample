# actix-web api server

Rust製のREST APIサーバー。LDAP認証とJWTトークンベースの認証を実装し、OpenTelemetryによる可観測性をサポートします。

- [actix-web api server](#actix-web-api-server)
  - [アーキテクチャ概要](#アーキテクチャ概要)
  - [技術スタック](#技術スタック)
  - [主な機能](#主な機能)
  - [module構造](#module構造)
  - [環境変数](#環境変数)
  - [起動方法(Docker)](#起動方法docker)
  - [起動方法(Dockerを使用しない)](#起動方法dockerを使用しない)
  - [開発方法](#開発方法)
  - [テスト実行方法](#テスト実行方法)
  - [openapi specification生成方法](#openapi-specification生成方法)
  - [開発ガイドライン](#開発ガイドライン)
  - [トラブルシューティング](#トラブルシューティング)
  - [TODO](#todo)

## アーキテクチャ概要

本APIサーバーは、レイヤードアーキテクチャを採用しています。

```
┌─────────────────────────────────────────┐
│      Presentation Layer                 │
│  (Actix-web Handlers + Middleware)      │
│  - JWT認証ミドルウェア                   │
│  - OpenTelemetryトレーシング             │
│  - CORS設定                             │
├─────────────────────────────────────────┤
│      Application Layer                  │
│     (Use Cases + Validation)            │
│  - ビジネスロジック                      │
│  - 入力バリデーション                    │
├─────────────────────────────────────────┤
│         Domain Layer                    │
│        (Models + Traits)                │
│  - ドメインモデル定義                    │
│  - 共通トレイト                         │
├─────────────────────────────────────────┤
│      Infrastructure Layer               │
│  (Diesel + LDAP + OpenTelemetry)        │
│  - データベースアクセス                  │
│  - LDAP認証                             │
│  - テレメトリエクスポート                │
└─────────────────────────────────────────┘
```

### 認証フロー

```
Client → POST /login → LDAP認証 → JWT発行 → Client
                          ↓
                    ユーザー情報をDB保存
                          
Client → GET /api/* → JWT検証 → ハンドラー実行
                        ↓
                   401 Unauthorized (失敗時)
```

### データフロー

```
HTTP Request
    ↓
Middleware (JWT認証、トレーシング)
    ↓
Handler (services/api/*.rs)
    ↓
Use Case (models/*/usecases.rs)
    ↓
Diesel ORM
    ↓
PostgreSQL
```

## 技術スタック

### コア技術

- **言語**: Rust (Edition 2021)
- **Webフレームワーク**: Actix-web 4.x
  - 高性能な非同期Webフレームワーク
  - ミドルウェアによる認証・ロギング
- **ORM**: Diesel 2.0
  - 型安全なクエリビルダー
  - マイグレーション管理
- **データベース**: PostgreSQL
  - r2d2によるコネクションプール管理

### 認証・セキュリティ

- **JWT**: jsonwebtoken
  - HS256アルゴリズムによるトークン署名
  - 有効期限7日間
- **LDAP**: ldap3
  - Active Directory統合
  - Simple Bind認証
- **バリデーション**: validator 0.16
  - 宣言的なバリデーションルール

### API仕様・ドキュメント

- **OpenAPI**: utoipa 3.x
  - コードからOpenAPI 3.0仕様を自動生成
  - Swagger UIによるインタラクティブなドキュメント
- **Swagger UI**: utoipa-swagger-ui
  - /swagger-ui/ でアクセス可能

### 可観測性 (Observability)

- **OpenTelemetry**: opentelemetry, opentelemetry-otlp
  - 分散トレーシング
  - メトリクス収集
  - OTLP形式でのエクスポート
- **ログ**: tracing, tracing-subscriber
  - 構造化ログ
  - OpenTelemetryとの統合

### 開発・テスト

- **テスト**: cargo test
  - ユニットテスト
  - 統合テスト (tests/)
- **ホットリロード**: cargo-watch
  - ファイル変更時の自動再起動

## 主な機能

### 認証機能

- **LDAP認証**: Active Directoryとの統合
- **JWT認証**: トークンベースのステートレス認証
- **グループフィルタリング**: 特定グループ(Partner)のログイン拒否

### API機能

- **ユーザー管理**: ユーザー一覧取得
- **顧客カテゴリ管理**: CRUD操作
- **バリデーション**: 入力データの検証
- **エラーハンドリング**: 統一されたエラーレスポンス

### 可観測性

- **分散トレーシング**: HTTPリクエスト、DBクエリ、認証処理のトレース
- **メトリクス**: リクエスト数、レスポンス時間、エラー率
- **構造化ログ**: JSON形式のログ出力

## module構造

| tree                              | 説明                                                                                           |
| --------------------------------- | ---------------------------------------------------------------------------------------------- |
| ── src                            |                                                                                                |
| ├── bin                           |                                                                                                |
| │  └── generate_openapi_schema.rs | # openapi specification 生成用のバイナリ                                                       |
| ├── config.rs                     | # 環境変数、.envファイルをConfig構造体へデシリアライズします。                                   |
| ├── errors.rs                     | # APIが発行するエラーを管理します                                   |
| ├── lib.rs                        | # DB接続の設定等を行うライブラリのトップレベルモジュールです                                   |
| ├── main.rs                       | # actix-webサーバを起動するトップレベルモジュールです                                          |
| ├── middleware.rs                 | # jwt認証などミドルウェア関連の定義を行います                                                  |
| │── models                        | # models配下のモジュールを置きます                                                             |
| │  ├── users                      | # 各モデル(例: users)配下のモジュールを置きます                                                |
| │  │  └── usecases.rs             | # 取得用・インサート用など個別の構造体(必要最低限)と実際にDBアクセスするメソッドを定義します。 |
| │  └── users.rs                   | # 各モデル(例: users)の構造体(全カラムの定義)を置いて置く予定です                              |
| ├── models.rs                     | # models配下のモジュール(各モデル)を宣言します。                                               |
| ├── schema.rs                     | # diesel migration runで自動生成されるDB操作用スキーマです。                                   |
| ├── services                      | # アクセスポイント用のモジュールを宣言します。                                                 |
| │  ├── api                        | # 認証が必要な物はapiモジュール配下に定義します                                                |
| │  ├── api.rs                     | # ミドルウェアで認証を入れています。apiディレクトリ配下のアクセスポイント定義をまとめます。    |
| │  └── auth.rs                    | # ログイン関係(認証が不要なもの)のアクセスポイントを定義します                                 |
| ├── services.rs                   | # アクセスポイントのスコープ毎にモジュールを宣言します。                                       |
| ├── swagger.rs                    | # swagger uiのアクセスポイントを定義しています。                                               |
| └── traits.rs                     | # traitを定義します。                                                                     |
| ── tests                          | # Integration Testを置きます                                                             |

## 環境変数

.envファイルか実行時環境変数に以下の値を設定してください

### データベース設定

- DATABASE_URL
  - 例: DATABASE_URL=postgres://postgres:P%40ssw0rd@localhost/development
- TEST_DATABASE_URL
  - 例: TEST_DATABASE_URL=postgres://postgres:P%40ssw0rd@localhost:5433/test

### 認証設定

- JWT_SECRET
  - 例: JWT_SECRET="18 A6 77 73 7F 72 44 6C 26 84 0B 19 75 E0 07 FA 73 A4 A8 82 21 C7 99 AC 0D C6 A5 FE D0 E4 E0 E6"
- LDAP_URI=ldap://ad.example.com
- LDAP_UID_COLUMN=cn
- LDAP_FILTER="(objectCategory=CN=Person*)"
- LDAP_USER_DN="cn=users,dc=example,dc=com"
- LDAP_GUARD_FILTER="(objectCategory=CN=Group*)"

### OpenTelemetry設定 (オプション)

OpenTelemetryによる分散トレーシングとメトリクス収集を有効化できます。

- OTEL_ENABLED
  - OpenTelemetryの有効/無効を制御します
  - デフォルト: false
  - 例: OTEL_ENABLED=true
  
- OTEL_ENDPOINT
  - OTLPエクスポーターのエンドポイントURL
  - デフォルト: http://localhost:4317
  - 例: OTEL_ENDPOINT=http://jaeger:4317
  
- OTEL_SERVICE_NAME
  - サービス名 (トレースに表示される名前)
  - デフォルト: rust-api
  - 例: OTEL_SERVICE_NAME=my-api-server
  
- OTEL_SERVICE_VERSION
  - サービスバージョン
  - デフォルト: 0.1.0
  - 例: OTEL_SERVICE_VERSION=1.0.0

#### OpenTelemetryバックエンド設定例

##### Jaegerを使用する場合

docker-composeでJaegerを起動:

```yaml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC receiver
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

環境変数設定:
```bash
OTEL_ENABLED=true
OTEL_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=rust-api
```

Jaeger UIにアクセス: http://localhost:16686

##### Grafana Tempoを使用する場合

docker-composeでTempoを起動:

```yaml
services:
  tempo:
    image: grafana/tempo:latest
    command: [ "-config.file=/etc/tempo.yaml" ]
    volumes:
      - ./tempo.yaml:/etc/tempo.yaml
    ports:
      - "4317:4317"    # OTLP gRPC
      - "3200:3200"    # Tempo HTTP
```

tempo.yaml設定例:
```yaml
server:
  http_listen_port: 3200

distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: 0.0.0.0:4317

storage:
  trace:
    backend: local
    local:
      path: /tmp/tempo/traces
```

環境変数設定:
```bash
OTEL_ENABLED=true
OTEL_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=rust-api
```

##### OpenTelemetry Collectorを使用する場合

より柔軟な設定が必要な場合は、OpenTelemetry Collectorを経由できます:

```yaml
services:
  otel-collector:
    image: otel/opentelemetry-collector:latest
    command: ["--config=/etc/otel-collector-config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml
    ports:
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
```

環境変数設定:
```bash
OTEL_ENABLED=true
OTEL_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=rust-api
```

## 起動方法(Docker)

1. Docker Desktopなど、docker, docker-composeが使える環境があるか確認してください
2. 以下のコマンドで起動します。  
    ```docker-compose -f docker-compose.yml -f docker-compose.services.yml```

## 起動方法(Dockerを使用しない)

1. rustをインストールします。<https://www.rust-lang.org/ja/tools/install>
2. localhost:5432ポートで起動するpostgresサーバを用意します。(docker-composeファイルでも可)
3. diesel_cliをインストールします  
    ```cargo install diesel_cli --no-default-features --features postgres```
4. dieselの初期化を行います。  
    ```diesel setup```
5. migrationを行います。  
    ```diesel migration run```
6. HMRを使用するためcargo-watchをインストールします  
    ```cargo install cargo-watch```
7. actix-webサーバを起動します  
    ```RUST_BACKTRACE=1 RUST_LOG=debug cargo watch -x run```

## 開発方法

1. vscodeを使用します。
2. rust-analyzer拡張機能を使用してください
3. apiの定義を追加した場合はutoipaによるschema定義とswagger.rsへの反映をします。
4. [http://localhost:8080/swagger-ui/]にアクセスしAPIが正常に動作するか実際に通信して確かめます。
5. [openapi specification生成方法](#openapi-specification生成方法)の生成を忘れずに行ってください。

## テスト実行方法

### 前提条件

テストを実行する前に、テスト用のPostgreSQLデータベースを起動してください。

```bash
# Dockerでテスト用データベースを起動
docker compose -f docker-compose.test.yml up -d

# データベースが起動したことを確認
docker ps | grep rust-api-test-db
```

### 基本的なテスト実行

全てのテストを実行:
```bash
cargo test
```

詳細なログ付きでテスト実行:
```bash
RUST_BACKTRACE=1 RUST_LOG=debug cargo test
```

特定のテストファイルのみ実行:
```bash
# JWT認証のテスト
cargo test --test jwt_auth

# 顧客カテゴリのテスト
cargo test --test customer_categories

# ユーザーエラーケースのテスト
cargo test --test users_error_cases

# 認証エラーケースのテスト
cargo test --test auth_error_cases

# LDAPモックのテスト
cargo test --test ldap_mock_tests
```

特定のテスト関数のみ実行:
```bash
cargo test test_jwt_auth_wrapper -- --nocapture
```

### テストカバレッジの計測

#### 初回セットアップ

```bash
# cargo-llvm-covのインストール
cargo install cargo-llvm-cov

# LLVMツールのインストール
rustup component add llvm-tools-preview
```

#### カバレッジレポートの生成

テキスト形式でカバレッジを表示:
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests -- --test-threads=1
```

HTML形式でカバレッジレポートを生成:
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests --html -- --test-threads=1
```

HTMLレポートは `target/llvm-cov/html/index.html` に生成されます。ブラウザで開いて確認できます:
```bash
# Linuxの場合
xdg-open target/llvm-cov/html/index.html

# macOSの場合
open target/llvm-cov/html/index.html

# Windowsの場合
start target/llvm-cov/html/index.html
```

JSON形式でカバレッジを出力（CI/CD用）:
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests --json --output-path coverage.json -- --test-threads=1
```

LCOV形式でカバレッジを出力（他のツールとの連携用）:
```bash
cargo llvm-cov --test jwt_auth --test customer_categories --test users_error_cases --test auth_error_cases --test ldap_mock_tests --lcov --output-path lcov.info -- --test-threads=1
```

### テストスイート概要

本プロジェクトには以下のテストスイートがあります（合計42テスト）:

| テストスイート | テスト数 | 説明 |
|--------------|---------|------|
| **jwt_auth** | 11 | JWT認証ミドルウェア、トレーシングミドルウェア、リクエストデータ作成のテスト |
| **customer_categories** | 8 | 顧客カテゴリAPIのエラーケース、バリデーション、認証テスト |
| **users_error_cases** | 6 | ユーザーAPIのエラーケース、ページネーションテスト |
| **auth_error_cases** | 7 | 認証エンドポイントのバリデーション、エラーハンドリングテスト |
| **ldap_mock_tests** | 10 | LDAP認証ロジック、JWT生成、ユーザー作成フローのテスト |

### カバレッジ目標

現在のカバレッジ状況:

- **全体カバレッジ**: 58.18%
- **ミドルウェア**: 87.29% ✅
- **データモデル**: 100% ✅
- **APIエンドポイント**: 73-83% ✅
- **認証サービス**: 27.56% ⚠️ (LDAP依存のため)

詳細なカバレッジレポートは `test-coverage-report.md` を参照してください。

### テストのベストプラクティス

#### 1. データベーステスト

テストでは自動的にマイグレーションが実行され、各テストで新しいデータベース状態が使用されます:

```rust
#[actix_web::test]
async fn test_insert_category() {
    let pool = rust_api::create_test_connection_pool();
    // テストコード
}
```

#### 2. 認証が必要なエンドポイントのテスト

JWT トークンを生成してテスト:

```rust
fn create_valid_token() -> String {
    let claims = UserClaims {
        id: 1,
        username: "testuser".into(),
        exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp()
    };
    // トークン生成
}

#[actix_web::test]
async fn test_protected_endpoint() {
    let token = create_valid_token();
    let req = test::TestRequest::get()
        .uri("/api/protected")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", token)))
        .to_request();
    // テスト実行
}
```

#### 3. エラーケースのテスト

バリデーションエラー、認証エラー、データベースエラーなど、様々なエラーケースをテスト:

```rust
#[actix_web::test]
async fn test_validation_error() {
    let data = NewCategoryBody {
        name: "a".repeat(256), // 長すぎる名前
    };
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400); // Bad Request
}
```

### CI/CDでのテスト実行

GitHub Actionsなどでテストを実行する場合の例:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:17-alpine
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test_db
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: llvm-tools-preview
      
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      
      - name: Run tests with coverage
        run: |
          cargo llvm-cov --test jwt_auth --test customer_categories \
            --test users_error_cases --test auth_error_cases \
            --test ldap_mock_tests --lcov --output-path lcov.info \
            -- --test-threads=1
        env:
          DATABASE_URL: postgres://test:test@localhost/test_db
          TEST_DATABASE_URL: postgres://test:test@localhost/test_db
      
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
```

### トラブルシューティング

#### テストデータベースに接続できない

```bash
# データベースが起動しているか確認
docker ps | grep rust-api-test-db

# データベースを再起動
docker compose -f docker-compose.test.yml down
docker compose -f docker-compose.test.yml up -d

# 接続テスト
psql postgres://test:test@localhost:5432/test_db -c "SELECT 1"
```

#### マイグレーションエラー

```bash
# テストデータベースをリセット
docker compose -f docker-compose.test.yml down -v
docker compose -f docker-compose.test.yml up -d
```

#### 特定のテストが失敗する

```bash
# 詳細なログを出力して実行
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_name -- --nocapture

# 単一スレッドで実行（並行実行の問題を回避）
cargo test -- --test-threads=1
```

## openapi specification生成方法

1. 以下のコマンドを実行します。  
    ```cargo run --bin generate_openapi_schema```
2. openapi_schema.json が出力されます

## 開発ガイドライン

### コーディング規約

#### 命名規則

- **関数・変数**: snake_case
  ```rust
  fn insert_new_user() { }
  let user_name = "example";
  ```

- **型・構造体・Enum**: PascalCase
  ```rust
  struct User { }
  enum ServiceError { }
  ```

- **定数**: SCREAMING_SNAKE_CASE
  ```rust
  const API_PREFIX: &str = "/api";
  ```

#### モジュール構成

- **models/**: ドメインモデルとビジネスロジック
  - `models/{entity}.rs`: 構造体定義
  - `models/{entity}/usecases.rs`: CRUD操作とビジネスロジック

- **services/**: APIエンドポイント
  - `services/auth.rs`: 認証不要なエンドポイント
  - `services/api/*.rs`: 認証必須なエンドポイント

#### エラーハンドリング

- `expect()` の使用を避け、`?` 演算子を使用する
  ```rust
  // ❌ 避けるべき
  let conn = pool.get().expect("couldn't get db connection");
  
  // ✅ 推奨
  let conn = pool.get()?;
  ```

- カスタムエラー型 `ServiceError` を使用
  ```rust
  pub enum ServiceError {
      InternalServerError,
      ValidationError { value: ValidationErrors }
  }
  ```

#### バリデーション

- `validator` クレートを使用した宣言的バリデーション
  ```rust
  #[derive(Validate)]
  pub struct CategoryValidator {
      #[validate(length(max = 255, message = "顧客分類は255文字以下で入力してください"))]
      pub name: String,
  }
  ```

- `IntoValidator` トレイトを実装
  ```rust
  impl IntoValidator<CategoryValidator> for CustomerCategory {
      fn validator(&self) -> CategoryValidator {
          CategoryValidator { name: self.name.clone() }
      }
  }
  ```

#### データベースアクセス

- Dieselの型安全なクエリビルダーを使用
  ```rust
  use crate::schema::users::dsl::*;
  
  users
      .filter(login_id.eq(user_login_id))
      .first::<User>(conn)
  ```

- トランザクションを適切に使用
  ```rust
  conn.transaction::<_, diesel::result::Error, _>(|conn| {
      // 複数の操作
      Ok(())
  })
  ```

### OpenTelemetryトレーシングの追加方法

#### 1. Use Case関数へのトレーシング追加

`#[instrument]` 属性を使用して、関数の実行をトレースします。

```rust
use tracing::instrument;

#[instrument(skip(conn), fields(db.operation = "insert_user"))]
pub fn insert_new_user(
    conn: &mut DbConnection,
    user: NewUser,
) -> QueryResult<User> {
    // 実装
}
```

**パラメータ説明**:
- `skip(conn)`: トレースに含めないパラメータ (DB接続など)
- `fields(...)`: カスタム属性の追加

#### 2. ハンドラーへのトレーシング追加

```rust
use tracing::{info, error};

#[utoipa::path(/* ... */)]
pub async fn get_users(pool: web::Data<DbPool>) -> Result<impl Responder, ServiceError> {
    info!("Fetching all users");
    
    let result = web::block(move || {
        let mut conn = pool.get()?;
        users::usecases::get_all_users(&mut conn)
    })
    .await?;
    
    info!("Successfully fetched {} users", result.len());
    Ok(web::Json(result))
}
```

#### 3. エラー時のトレーシング

```rust
match some_operation() {
    Ok(result) => {
        tracing::info!("Operation succeeded");
        result
    }
    Err(e) => {
        tracing::error!("Operation failed: {:?}", e);
        return Err(ServiceError::InternalServerError);
    }
}
```

#### 4. カスタムスパンの作成

より詳細なトレーシングが必要な場合:

```rust
use tracing::{info_span, Instrument};

async fn complex_operation() {
    let span = info_span!("ldap_authentication", user = %username);
    
    async {
        // LDAP認証処理
        info!("Binding to LDAP server");
        // ...
        info!("Searching user in LDAP");
        // ...
    }
    .instrument(span)
    .await
}
```

### API追加の手順

1. **モデル定義** (`models/{entity}.rs`)
   ```rust
   #[derive(Queryable, Serialize, ToSchema)]
   pub struct MyEntity {
       pub id: i32,
       pub name: String,
   }
   ```

2. **Use Case実装** (`models/{entity}/usecases.rs`)
   ```rust
   #[instrument(skip(conn))]
   pub fn get_all(conn: &mut DbConnection) -> QueryResult<Vec<MyEntity>> {
       use crate::schema::my_entities::dsl::*;
       my_entities.load::<MyEntity>(conn)
   }
   ```

3. **ハンドラー実装** (`services/api/{entity}.rs`)
   ```rust
   #[utoipa::path(
       get,
       path = "/api/my-entities",
       responses(
           (status = 200, description = "Success", body = [MyEntity])
       )
   )]
   pub async fn get_all(pool: web::Data<DbPool>) -> Result<impl Responder, ServiceError> {
       // 実装
   }
   ```

4. **Swagger定義に追加** (`swagger.rs`)
   ```rust
   #[derive(OpenApi)]
   #[openapi(
       paths(
           services::api::my_entities::get_all,
       ),
       components(schemas(MyEntity))
   )]
   struct ApiDoc;
   ```

5. **ルーティング登録** (`services/api.rs` または `main.rs`)
   ```rust
   .service(
       web::scope("/api")
           .service(my_entities::get_all)
   )
   ```

6. **テスト作成** (`tests/my_entities.rs`)
   ```rust
   #[actix_web::test]
   async fn test_get_all() {
       // テスト実装
   }
   ```

### テストのベストプラクティス

- **トランザクションテスト**: 自動ロールバックを使用
  ```rust
  conn.test_transaction::<_, ServiceError, _>(|conn| {
      let result = insert_new_entity(conn, data)?;
      assert_eq!(result.name, "expected");
      Ok(())
  })
  ```

- **統合テスト**: 実際のHTTPリクエストをシミュレート
  ```rust
  let app = test::init_service(
      App::new()
          .app_data(web::Data::new(pool.clone()))
          .service(my_handler)
  ).await;
  
  let req = test::TestRequest::get()
      .uri("/api/endpoint")
      .to_request();
  
  let resp = test::call_service(&app, req).await;
  assert!(resp.status().is_success());
  ```

## トラブルシューティング

### よくある問題と解決方法

#### 1. データベース接続エラー

**症状**:
```
Error: couldn't get db connection from pool
```

**原因**:
- PostgreSQLが起動していない
- DATABASE_URLが正しく設定されていない
- コネクションプール枯渇

**解決方法**:
```bash
# PostgreSQLの起動確認
docker ps | grep postgres

# 環境変数の確認
echo $DATABASE_URL

# .envファイルの確認
cat .env

# データベース接続テスト
psql $DATABASE_URL -c "SELECT 1"
```

#### 2. マイグレーションエラー

**症状**:
```
Error: Migration xxx has already been run
```

**解決方法**:
```bash
# マイグレーション状態の確認
diesel migration list

# マイグレーションのロールバック
diesel migration revert

# 再度マイグレーション実行
diesel migration run
```

#### 3. JWT認証エラー

**症状**:
```
401 Unauthorized
```

**原因**:
- トークンが無効または期限切れ
- JWT_SECRETが正しく設定されていない
- Authorizationヘッダーの形式が不正

**解決方法**:
```bash
# JWT_SECRETの確認
echo $JWT_SECRET

# トークンの確認 (jwt.ioでデコード)
# Authorizationヘッダーの形式: Bearer <token>

# ログで詳細確認
RUST_LOG=debug cargo run
```

#### 4. LDAP認証エラー

**症状**:
```
LDAP bind failed
```

**原因**:
- LDAPサーバーに接続できない
- 認証情報が不正
- LDAP設定が間違っている

**解決方法**:
```bash
# LDAP接続テスト
ldapsearch -H $LDAP_URI -D "cn=user,dc=example,dc=com" -W

# 環境変数の確認
echo $LDAP_URI
echo $LDAP_USER_DN
echo $LDAP_FILTER

# デバッグログで詳細確認
RUST_LOG=debug cargo run
```

#### 5. OpenTelemetryエクスポートエラー

**症状**:
```
OpenTelemetry trace error occurred
```

**原因**:
- OTLPエンドポイントに接続できない
- Jaeger/Tempoが起動していない

**解決方法**:
```bash
# バックエンドの起動確認
docker ps | grep jaeger

# エンドポイントの確認
echo $OTEL_ENDPOINT

# OpenTelemetryを無効化して起動
OTEL_ENABLED=false cargo run

# ネットワーク接続確認
curl -v http://localhost:4317
```

#### 6. コンパイルエラー

**症状**:
```
error[E0433]: failed to resolve: use of undeclared crate or module
```

**解決方法**:
```bash
# 依存関係の更新
cargo update

# クリーンビルド
cargo clean
cargo build

# Cargo.lockの削除と再生成
rm Cargo.lock
cargo build
```

#### 7. テスト失敗

**症状**:
```
test result: FAILED
```

**解決方法**:
```bash
# テストデータベースの確認
echo $TEST_DATABASE_URL

# テストデータベースのリセット
diesel database reset --database-url $TEST_DATABASE_URL

# 詳細ログ付きでテスト実行
RUST_BACKTRACE=1 RUST_LOG=debug cargo test -- --nocapture

# 特定のテストのみ実行
cargo test test_name -- --nocapture
```

#### 8. パフォーマンス問題

**症状**:
- レスポンスが遅い
- タイムアウトが発生

**解決方法**:
```bash
# OpenTelemetryでトレース確認
# Jaeger UIで遅いクエリを特定

# データベースクエリの最適化
# EXPLAIN ANALYZEで実行計画確認

# コネクションプール設定の調整
# Cargo.tomlのr2d2設定を確認

# OpenTelemetryのオーバーヘッド確認
OTEL_ENABLED=false cargo run  # 無効時と比較
```

### ログレベルの設定

開発時は詳細なログを出力:
```bash
RUST_LOG=debug cargo run
```

本番環境では警告以上のみ:
```bash
RUST_LOG=warn cargo run
```

モジュール別のログレベル設定:
```bash
RUST_LOG=actix_web=info,diesel=debug,my_app=trace cargo run
```

### デバッグのヒント

1. **RUST_BACKTRACE**: スタックトレースを表示
   ```bash
   RUST_BACKTRACE=1 cargo run
   ```

2. **cargo-expand**: マクロ展開を確認
   ```bash
   cargo install cargo-expand
   cargo expand
   ```

3. **Swagger UI**: APIの動作確認
   - http://localhost:8080/swagger-ui/

4. **Jaeger UI**: トレースの確認
   - http://localhost:16686

## OpenTelemetry動作確認

OpenTelemetry統合の動作確認とパフォーマンステストについては、以下のドキュメントを参照してください：

### クイックスタート

```bash
# 自動テストスクリプトで全ての確認を実行
./test-otel.sh

# メトリクス収集の確認
./verify-metrics.sh

# パフォーマンス比較ベンチマーク
./benchmark-otel.sh
```

### 詳細ドキュメント

- **[OTEL_TESTING.md](OTEL_TESTING.md)** - OpenTelemetry動作確認の包括的ガイド
- **[MANUAL_OTEL_TEST.md](MANUAL_OTEL_TEST.md)** - 手動テスト手順
- **[METRICS_SETUP.md](METRICS_SETUP.md)** - メトリクス収集とPrometheus統合
- **[PERFORMANCE_TEST.md](PERFORMANCE_TEST.md)** - パフォーマンステストガイド
- **[OTEL_VERIFICATION_SUMMARY.md](OTEL_VERIFICATION_SUMMARY.md)** - 動作確認完了サマリー

### 主要なURL

- **Jaeger UI**: http://localhost:16686 - トレースの確認
- **Swagger UI**: http://localhost:8080/swagger-ui/ - API仕様とテスト
- **API Server**: http://localhost:8080

## TODO

  tag, context_pathに定数を指定する。 <https://github.com/juhaku/utoipa/issues/518>
