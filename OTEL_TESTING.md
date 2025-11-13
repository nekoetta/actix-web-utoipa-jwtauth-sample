# OpenTelemetry動作確認ガイド

このドキュメントでは、OpenTelemetry統合の動作確認方法を説明します。

## 目次

1. [前提条件](#前提条件)
2. [自動テストスクリプトの使用](#自動テストスクリプトの使用)
3. [手動テスト手順](#手動テスト手順)
4. [トレースの確認方法](#トレースの確認方法)
5. [メトリクスの確認方法](#メトリクスの確認方法)
6. [パフォーマンステスト](#パフォーマンステスト)
7. [トラブルシューティング](#トラブルシューティング)

## 前提条件

以下のツールがインストールされていることを確認してください：

- Docker & Docker Compose
- Rust (最新安定版)
- diesel_cli: `cargo install diesel_cli --no-default-features --features postgres`
- curl (HTTPリクエスト用)

## 自動テストスクリプトの使用

最も簡単な方法は、提供されているテストスクリプトを使用することです。

```bash
./test-otel.sh
```

このスクリプトは以下を自動的に実行します：

1. Jaeger と PostgreSQL の起動
2. データベースマイグレーション
3. APIサーバーのビルドと起動（OpenTelemetry有効）
4. テストAPIコールの実行
5. Jaegerでのトレース確認

スクリプト実行後、以下のURLにアクセスできます：

- **Jaeger UI**: http://localhost:16686
- **Swagger UI**: http://localhost:8080/swagger-ui/
- **API Server**: http://localhost:8080

## 手動テスト手順

### ステップ 1: Jaeger と PostgreSQL の起動

```bash
docker compose -f docker-compose.otel.yml up -d
```

起動確認：

```bash
# Jaeger UI にアクセス
curl http://localhost:16686

# PostgreSQL の確認
docker exec rust-api-postgres pg_isready -U test
```

### ステップ 2: データベースマイグレーション

```bash
diesel migration run
```

### ステップ 3: 環境変数の設定

`.env` ファイルを編集するか、環境変数を設定します：

```bash
export OTEL_ENABLED=true
export OTEL_ENDPOINT=http://localhost:4317
export OTEL_SERVICE_NAME=rust-api-test
export OTEL_SERVICE_VERSION=1.0.0
export RUST_LOG=info
```

### ステップ 4: APIサーバーの起動

```bash
cargo run --release
```

または開発モードで：

```bash
RUST_LOG=debug cargo run
```

### ステップ 5: APIコールの実行

#### 5.1 Swagger UI を使用

1. ブラウザで http://localhost:8080/swagger-ui/ を開く
2. 各エンドポイントを試す（認証が必要なものは401エラーになります）
3. トレースが生成されます

#### 5.2 curl を使用

```bash
# ユーザー一覧取得（認証なし - 401エラー）
curl -v http://localhost:8080/api/users/

# 顧客カテゴリ一覧取得（認証なし - 401エラー）
curl -v http://localhost:8080/api/customers/categories

# OpenAPI仕様取得（認証不要 - 200 OK）
curl http://localhost:8080/api-doc/openapi.json

# Swagger UI（認証不要 - 200 OK）
curl http://localhost:8080/swagger-ui/
```

#### 5.3 認証付きAPIコール

まず、テストユーザーでログイン（LDAPサーバーが必要）：

```bash
# ログイン（LDAPサーバーが起動している場合）
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"testpass"}'
```

レスポンスから `Authorization` ヘッダーのトークンを取得し、使用：

```bash
# トークンを変数に保存
TOKEN="Bearer eyJ0eXAiOiJKV1QiLCJhbGc..."

# 認証付きでユーザー一覧取得
curl -H "Authorization: $TOKEN" http://localhost:8080/api/users/

# 認証付きで顧客カテゴリ作成
curl -X POST http://localhost:8080/api/customers/categories \
  -H "Authorization: $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"テストカテゴリ"}'
```

## トレースの確認方法

### Jaeger UI でのトレース確認

1. **Jaeger UI を開く**: http://localhost:16686

2. **サービスを選択**:
   - Service ドロップダウンから `rust-api-test` を選択

3. **トレースを検索**:
   - "Find Traces" ボタンをクリック

4. **トレースの詳細を確認**:
   - トレース一覧から任意のトレースをクリック
   - スパンの階層構造を確認
   - 各スパンの実行時間を確認

### 確認すべきトレース情報

#### HTTPリクエストトレース

各HTTPリクエストには以下の情報が含まれます：

- **http.method**: GET, POST, PUT, DELETE
- **http.target**: リクエストパス（例: /api/users/）
- **http.status_code**: HTTPステータスコード
- **http.user_agent**: ユーザーエージェント
- **request_id**: リクエストID（UUID）

#### データベースクエリトレース

データベース操作には以下の情報が含まれます：

- **db.operation**: 操作種別（insert_user, search_user, get_all_users など）
- **db.user**: 対象ユーザー（該当する場合）
- **db.category**: 対象カテゴリ（該当する場合）

#### 認証トレース

認証処理には以下の情報が含まれます：

- **auth.token_valid**: トークンの有効性（true/false）
- **auth.ldap_bind**: LDAP Bind操作
- **auth.user_search**: LDAPユーザー検索

### トレースの例

```
HTTP Request [200ms]
├─ JWT Validation [10ms]
│  └─ Token Decode [2ms]
├─ Database Query [150ms]
│  ├─ Get Connection [5ms]
│  └─ Execute Query [145ms]
└─ Response Serialization [40ms]
```

## メトリクスの確認方法

### 収集されるメトリクス

現在の実装では、以下のメトリクスが収集されます：

#### 1. HTTPメトリクス

- **http_requests_total**: リクエスト総数
  - ラベル: method, path, status
- **http_request_duration_seconds**: リクエスト処理時間
  - ラベル: method, path
- **http_requests_in_flight**: 同時実行リクエスト数

#### 2. データベースメトリクス

- **db_queries_total**: クエリ総数
  - ラベル: operation
- **db_query_duration_seconds**: クエリ実行時間
  - ラベル: operation
- **db_connection_pool_size**: コネクションプール使用数

#### 3. 認証メトリクス

- **auth_attempts_total**: 認証試行回数
  - ラベル: success/failure
- **jwt_validations_total**: JWT検証回数
  - ラベル: valid/invalid

### Prometheusエクスポーターの設定（オプション）

メトリクスをPrometheusで収集する場合は、以下の設定を追加します：

#### docker-compose.otel.yml に追加

```yaml
  prometheus:
    image: prom/prometheus:latest
    container_name: rust-api-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    networks:
      - otel-network
```

#### prometheus.yml の作成

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'rust-api'
    static_configs:
      - targets: ['host.docker.internal:8080']
```

#### メトリクスエンドポイントの追加

現在の実装では、メトリクスはOpenTelemetry経由でエクスポートされます。
Prometheusエクスポーターを追加する場合は、`src/metrics.rs` を拡張してください。

## パフォーマンステスト

### OpenTelemetry有効時と無効時の比較

#### 1. OpenTelemetry無効でのベンチマーク

```bash
# OpenTelemetryを無効化
export OTEL_ENABLED=false
cargo run --release

# 別のターミナルでベンチマーク実行
# Apache Bench を使用
ab -n 1000 -c 10 http://localhost:8080/swagger-ui/

# または wrk を使用
wrk -t4 -c100 -d30s http://localhost:8080/swagger-ui/
```

結果を記録します（例）：

```
Requests per second:    5000.00 [#/sec]
Time per request:       2.000 [ms]
```

#### 2. OpenTelemetry有効でのベンチマーク

```bash
# OpenTelemetryを有効化
export OTEL_ENABLED=true
export OTEL_ENDPOINT=http://localhost:4317
cargo run --release

# 同じベンチマークを実行
ab -n 1000 -c 10 http://localhost:8080/swagger-ui/
```

結果を記録します（例）：

```
Requests per second:    4800.00 [#/sec]
Time per request:       2.083 [ms]
```

#### 3. オーバーヘッドの計算

```
オーバーヘッド = (有効時の時間 - 無効時の時間) / 無効時の時間 × 100%
             = (2.083 - 2.000) / 2.000 × 100%
             = 4.15%
```

### 期待されるパフォーマンス

- **オーバーヘッド**: 5%未満が理想
- **レイテンシ増加**: 1-2ms程度
- **スループット低下**: 5%未満

### 負荷テストツール

#### Apache Bench (ab)

```bash
# インストール（Ubuntu/Debian）
sudo apt-get install apache2-utils

# 基本的な使用方法
ab -n 1000 -c 10 http://localhost:8080/api-doc/openapi.json
```

#### wrk

```bash
# インストール（Ubuntu/Debian）
sudo apt-get install wrk

# 基本的な使用方法
wrk -t4 -c100 -d30s http://localhost:8080/api-doc/openapi.json
```

#### hey

```bash
# インストール
go install github.com/rakyll/hey@latest

# 基本的な使用方法
hey -n 1000 -c 10 http://localhost:8080/api-doc/openapi.json
```

### パフォーマンステスト結果の記録

テスト結果を記録するためのテンプレート：

```markdown
## パフォーマンステスト結果

### テスト環境
- CPU: [CPU情報]
- メモリ: [メモリ容量]
- OS: [OS情報]
- Rust バージョン: [バージョン]

### OpenTelemetry無効時
- Requests/sec: [値]
- Latency (avg): [値]ms
- Latency (p95): [値]ms
- Latency (p99): [値]ms

### OpenTelemetry有効時
- Requests/sec: [値]
- Latency (avg): [値]ms
- Latency (p95): [値]ms
- Latency (p99): [値]ms

### オーバーヘッド
- スループット低下: [値]%
- レイテンシ増加: [値]ms
- 結論: [許容範囲内/要改善]
```

## トラブルシューティング

### 問題 1: Jaegerにトレースが表示されない

**症状**: APIコールを実行してもJaegerにトレースが表示されない

**確認事項**:

1. OpenTelemetryが有効になっているか確認
   ```bash
   echo $OTEL_ENABLED  # true であるべき
   ```

2. エンドポイントが正しいか確認
   ```bash
   echo $OTEL_ENDPOINT  # http://localhost:4317 であるべき
   ```

3. Jaegerが起動しているか確認
   ```bash
   docker ps | grep jaeger
   curl http://localhost:16686
   ```

4. APIサーバーのログを確認
   ```bash
   # エラーメッセージを探す
   RUST_LOG=debug cargo run
   ```

5. ネットワーク接続を確認
   ```bash
   # Jaegerのポートが開いているか確認
   nc -zv localhost 4317
   ```

**解決方法**:

- Jaegerを再起動: `docker compose -f docker-compose.otel.yml restart jaeger`
- APIサーバーを再起動
- トレースのエクスポートに時間がかかる場合があるので、数秒待つ

### 問題 2: APIサーバーが起動しない

**症状**: OpenTelemetry有効時にAPIサーバーが起動しない

**確認事項**:

1. エラーメッセージを確認
   ```bash
   RUST_BACKTRACE=1 RUST_LOG=debug cargo run
   ```

2. 依存関係を確認
   ```bash
   cargo update
   cargo build
   ```

3. OpenTelemetryを無効化して起動できるか確認
   ```bash
   OTEL_ENABLED=false cargo run
   ```

**解決方法**:

- `Cargo.lock` を削除して再ビルド
- OpenTelemetryの設定を確認
- エラーログから具体的な問題を特定

### 問題 3: パフォーマンスが著しく低下する

**症状**: OpenTelemetry有効時にパフォーマンスが10%以上低下する

**確認事項**:

1. サンプリングレートを確認（将来の実装）
2. バッチエクスポート設定を確認
3. ログレベルを確認（DEBUGは遅い）
   ```bash
   export RUST_LOG=info  # または warn
   ```

**解決方法**:

- サンプリングレートを調整（例: 10%のリクエストのみトレース）
- バッチサイズを増やす
- 本番環境ではログレベルを `warn` または `error` に設定

### 問題 4: データベース接続エラー

**症状**: データベースに接続できない

**確認事項**:

1. PostgreSQLが起動しているか確認
   ```bash
   docker ps | grep postgres
   ```

2. DATABASE_URLが正しいか確認
   ```bash
   echo $DATABASE_URL
   ```

3. マイグレーションが実行されているか確認
   ```bash
   diesel migration list
   ```

**解決方法**:

```bash
# PostgreSQLを再起動
docker compose -f docker-compose.otel.yml restart postgres

# マイグレーションを実行
diesel migration run

# 接続テスト
psql $DATABASE_URL -c "SELECT 1"
```

### 問題 5: メモリ使用量が増加する

**症状**: OpenTelemetry有効時にメモリ使用量が増加する

**確認事項**:

1. メモリ使用量を監視
   ```bash
   # プロセスのメモリ使用量を確認
   ps aux | grep rust-api
   
   # または htop を使用
   htop
   ```

2. トレースのバッファサイズを確認

**解決方法**:

- バッチエクスポートの頻度を増やす
- トレースのバッファサイズを減らす
- サンプリングレートを下げる

## ベストプラクティス

### 開発環境

- OpenTelemetryを有効化してローカルでテスト
- Jaeger UIでトレースを確認しながら開発
- ログレベルは `debug` で詳細情報を確認

### ステージング環境

- OpenTelemetryを有効化
- サンプリングレート: 100%（全リクエストをトレース）
- ログレベル: `info`

### 本番環境

- OpenTelemetryを有効化
- サンプリングレート: 1-10%（負荷に応じて調整）
- ログレベル: `warn` または `error`
- アラート設定: エラー率、レイテンシ閾値

## 参考リンク

- [OpenTelemetry公式ドキュメント](https://opentelemetry.io/docs/)
- [Jaeger公式ドキュメント](https://www.jaegertracing.io/docs/)
- [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry/)
- [opentelemetry-otlp](https://docs.rs/opentelemetry-otlp/)

## まとめ

このガイドに従って、OpenTelemetry統合の動作確認を行ってください。

1. ✅ 自動テストスクリプトを実行
2. ✅ Jaeger UIでトレースを確認
3. ✅ Swagger UIからAPIを実行
4. ✅ パフォーマンステストを実行
5. ✅ 結果を記録

問題が発生した場合は、トラブルシューティングセクションを参照してください。
