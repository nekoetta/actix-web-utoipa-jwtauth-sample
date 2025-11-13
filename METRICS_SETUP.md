# メトリクス収集セットアップガイド

このドキュメントでは、OpenTelemetryメトリクスの収集、確認、および可視化の方法を説明します。

## 目次

1. [メトリクス概要](#メトリクス概要)
2. [実装済みメトリクス](#実装済みメトリクス)
3. [メトリクスの確認方法](#メトリクスの確認方法)
4. [Prometheus統合（オプション）](#prometheus統合オプション)
5. [Grafanaダッシュボード（オプション）](#grafanaダッシュボードオプション)
6. [メトリクスの活用例](#メトリクスの活用例)

## メトリクス概要

本APIサーバーでは、以下の3つのカテゴリのメトリクスを収集しています：

1. **HTTPメトリクス**: リクエスト数、レスポンス時間、同時実行数
2. **データベースメトリクス**: クエリ数、実行時間、コネクションプール状態
3. **認証メトリクス**: 認証試行回数、JWT検証回数

これらのメトリクスは、OpenTelemetry経由で収集され、OTLP形式でエクスポートされます。

## 実装済みメトリクス

### HTTPメトリクス

#### 1. http_requests_total (Counter)

**説明**: HTTPリクエストの総数

**ラベル**:
- `method`: HTTPメソッド（GET, POST, PUT, DELETE）
- `path`: リクエストパス（/api/users/, /api/customers/categories など）
- `status`: HTTPステータスコード（200, 401, 404, 500 など）

**使用例**:
```rust
HttpMetrics::record_request("GET", "/api/users/", 200);
```

**活用方法**:
- エンドポイント別のリクエスト数を監視
- エラー率（4xx, 5xx）の計算
- トラフィックパターンの分析

#### 2. http_request_duration_seconds (Histogram)

**説明**: HTTPリクエストの処理時間（秒）

**ラベル**:
- `method`: HTTPメソッド
- `path`: リクエストパス

**使用例**:
```rust
let timer = DurationTimer::new();
// ... リクエスト処理 ...
HttpMetrics::record_duration("GET", "/api/users/", timer.elapsed_secs());
```

**活用方法**:
- レスポンスタイムの監視
- パフォーマンスボトルネックの特定
- SLA（Service Level Agreement）の監視

#### 3. http_requests_in_flight (UpDownCounter)

**説明**: 現在処理中のHTTPリクエスト数

**使用例**:
```rust
HttpMetrics::increment_in_flight();
// ... リクエスト処理 ...
HttpMetrics::decrement_in_flight();
```

**活用方法**:
- サーバー負荷の監視
- スケーリングの判断材料
- キャパシティプランニング

### データベースメトリクス

#### 4. db_queries_total (Counter)

**説明**: データベースクエリの総数

**ラベル**:
- `operation`: 操作種別（insert_user, search_user, get_all_users など）

**使用例**:
```rust
DbMetrics::record_query("insert_user");
```

**活用方法**:
- データベース負荷の監視
- クエリパターンの分析
- N+1問題の検出

#### 5. db_query_duration_seconds (Histogram)

**説明**: データベースクエリの実行時間（秒）

**ラベル**:
- `operation`: 操作種別

**使用例**:
```rust
let timer = DurationTimer::new();
// ... クエリ実行 ...
DbMetrics::record_duration("insert_user", timer.elapsed_secs());
```

**活用方法**:
- 遅いクエリの特定
- インデックスの効果測定
- データベースパフォーマンスの最適化

#### 6. db_connection_pool_size / db_connection_pool_idle (Gauge)

**説明**: コネクションプールの状態

**使用例**:
```rust
DbMetrics::record_pool_state(pool_size, idle_connections);
```

**活用方法**:
- コネクションプール枯渇の検出
- プールサイズの最適化
- データベース接続の監視

### 認証メトリクス

#### 7. auth_attempts_total (Counter)

**説明**: 認証試行の総数

**ラベル**:
- `result`: 結果（success / failure）

**使用例**:
```rust
AuthMetrics::record_attempt(true);  // 成功
AuthMetrics::record_attempt(false); // 失敗
```

**活用方法**:
- 認証失敗率の監視
- ブルートフォース攻撃の検出
- ユーザーログインパターンの分析

#### 8. jwt_validations_total (Counter)

**説明**: JWTトークン検証の総数

**ラベル**:
- `result`: 結果（valid / invalid）

**使用例**:
```rust
AuthMetrics::record_jwt_validation(true);  // 有効
AuthMetrics::record_jwt_validation(false); // 無効
```

**活用方法**:
- トークン有効性の監視
- 不正アクセスの検出
- トークン有効期限の最適化

## メトリクスの確認方法

### 方法1: 自動検証スクリプト

最も簡単な方法は、提供されているスクリプトを使用することです：

```bash
./verify-metrics.sh
```

このスクリプトは以下を実行します：
1. サービスの起動確認
2. テストトラフィックの生成
3. メトリクス収集の確認
4. 結果の表示

### 方法2: Jaeger UIでの確認

メトリクスはトレースデータに埋め込まれているため、Jaeger UIで確認できます：

1. Jaeger UIを開く: http://localhost:16686
2. サービス `rust-api` を選択
3. トレースを検索
4. 各スパンの実行時間を確認

**確認できる情報**:
- HTTPリクエストの処理時間
- データベースクエリの実行時間
- 認証処理の時間
- エラー発生状況

### 方法3: ログでの確認

デバッグモードでサーバーを起動すると、メトリクス情報がログに出力されます：

```bash
RUST_LOG=debug cargo run
```

ログ出力例：
```
[INFO] HTTP request: method=GET path=/api/users/ status=401 duration=0.015s
[INFO] JWT validation: result=invalid
[INFO] DB query: operation=search_user duration=0.005s
```

## Prometheus統合（オプション）

より詳細なメトリクス分析が必要な場合は、Prometheusを統合できます。

### ステップ1: Prometheusエクスポーターの追加

`Cargo.toml` に依存関係を追加：

```toml
[dependencies]
opentelemetry-prometheus = "0.16"
prometheus = "0.13"
```

### ステップ2: メトリクスエンドポイントの実装

`src/main.rs` にメトリクスエンドポイントを追加：

```rust
use actix_web::{web, HttpResponse};
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler() -> HttpResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(buffer)
}

// main関数内で登録
HttpServer::new(move || {
    App::new()
        .route("/metrics", web::get().to(metrics_handler))
        // ... 他のルート ...
})
```

### ステップ3: Prometheusの起動

`docker-compose.otel.yml` にPrometheusを追加：

```yaml
  prometheus:
    image: prom/prometheus:latest
    container_name: rust-api-prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    networks:
      - otel-network
```

### ステップ4: Prometheus設定ファイルの作成

`prometheus.yml` を作成：

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'rust-api'
    static_configs:
      - targets: ['host.docker.internal:8080']
    metrics_path: '/metrics'
```

### ステップ5: Prometheusの起動と確認

```bash
# Prometheusを起動
docker compose -f docker-compose.otel.yml up -d prometheus

# Prometheus UIにアクセス
open http://localhost:9090
```

### PromQLクエリ例

Prometheus UIで以下のクエリを実行できます：

#### リクエスト数（1分あたり）
```promql
rate(http_requests_total[1m])
```

#### エンドポイント別のリクエスト数
```promql
sum by (path) (http_requests_total)
```

#### エラー率（4xx, 5xx）
```promql
sum(rate(http_requests_total{status=~"4..|5.."}[5m])) 
/ 
sum(rate(http_requests_total[5m]))
```

#### 平均レスポンスタイム
```promql
rate(http_request_duration_seconds_sum[5m]) 
/ 
rate(http_request_duration_seconds_count[5m])
```

#### P95レスポンスタイム
```promql
histogram_quantile(0.95, 
  rate(http_request_duration_seconds_bucket[5m])
)
```

#### データベースクエリ数（1分あたり）
```promql
rate(db_queries_total[1m])
```

#### 認証失敗率
```promql
sum(rate(auth_attempts_total{result="failure"}[5m])) 
/ 
sum(rate(auth_attempts_total[5m]))
```

## Grafanaダッシュボード（オプション）

メトリクスを視覚的に監視するために、Grafanaを使用できます。

### ステップ1: Grafanaの起動

`docker-compose.otel.yml` にGrafanaを追加：

```yaml
  grafana:
    image: grafana/grafana:latest
    container_name: rust-api-grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana-storage:/var/lib/grafana
    networks:
      - otel-network

volumes:
  grafana-storage:
```

起動：

```bash
docker compose -f docker-compose.otel.yml up -d grafana
```

### ステップ2: Grafanaの設定

1. Grafana UIにアクセス: http://localhost:3000
2. ログイン: admin / admin
3. データソースを追加:
   - Configuration → Data Sources → Add data source
   - Prometheus を選択
   - URL: http://prometheus:9090
   - Save & Test

### ステップ3: ダッシュボードの作成

#### パネル1: リクエスト数（時系列）

- Query: `rate(http_requests_total[1m])`
- Visualization: Time series
- Legend: `{{method}} {{path}} {{status}}`

#### パネル2: レスポンスタイム（時系列）

- Query: `histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))`
- Visualization: Time series
- Legend: P95 Response Time

#### パネル3: エラー率（ゲージ）

- Query: `sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m]))`
- Visualization: Gauge
- Thresholds: 0.01 (warning), 0.05 (critical)

#### パネル4: データベースクエリ数（時系列）

- Query: `rate(db_queries_total[1m])`
- Visualization: Time series
- Legend: `{{operation}}`

#### パネル5: 認証失敗率（ゲージ）

- Query: `sum(rate(auth_attempts_total{result="failure"}[5m])) / sum(rate(auth_attempts_total[5m]))`
- Visualization: Gauge
- Thresholds: 0.1 (warning), 0.3 (critical)

### ダッシュボードのエクスポート/インポート

ダッシュボードをJSON形式でエクスポート/インポートできます：

1. Dashboard settings → JSON Model
2. JSONをコピーして保存
3. 他の環境でインポート

## メトリクスの活用例

### 1. パフォーマンス監視

**目的**: レスポンスタイムの劣化を検出

**メトリクス**:
- `http_request_duration_seconds`
- `db_query_duration_seconds`

**アラート条件**:
```promql
# P95レスポンスタイムが1秒を超えた場合
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1.0
```

### 2. エラー監視

**目的**: エラー率の上昇を検出

**メトリクス**:
- `http_requests_total{status=~"5.."}`

**アラート条件**:
```promql
# 5xxエラー率が5%を超えた場合
sum(rate(http_requests_total{status=~"5.."}[5m])) 
/ 
sum(rate(http_requests_total[5m])) > 0.05
```

### 3. セキュリティ監視

**目的**: 不正アクセスの検出

**メトリクス**:
- `auth_attempts_total{result="failure"}`
- `jwt_validations_total{result="invalid"}`

**アラート条件**:
```promql
# 認証失敗率が30%を超えた場合（ブルートフォース攻撃の可能性）
sum(rate(auth_attempts_total{result="failure"}[5m])) 
/ 
sum(rate(auth_attempts_total[5m])) > 0.3
```

### 4. キャパシティプランニング

**目的**: スケーリングの必要性を判断

**メトリクス**:
- `http_requests_in_flight`
- `db_connection_pool_size`
- `db_connection_pool_idle`

**アラート条件**:
```promql
# コネクションプールの使用率が90%を超えた場合
(db_connection_pool_size - db_connection_pool_idle) 
/ 
db_connection_pool_size > 0.9
```

### 5. ビジネスメトリクス

**目的**: ビジネス指標の追跡

**メトリクス**:
- `http_requests_total{path="/api/customers/categories", method="POST"}` (新規カテゴリ作成数)
- `auth_attempts_total{result="success"}` (ログイン数)

**分析例**:
```promql
# 1日あたりの新規カテゴリ作成数
sum(increase(http_requests_total{path="/api/customers/categories", method="POST", status="200"}[1d]))
```

## ベストプラクティス

### 1. メトリクスの命名規則

- **Counter**: `_total` サフィックスを使用（例: `http_requests_total`）
- **Histogram**: `_seconds` または `_bytes` サフィックスを使用
- **Gauge**: 現在の状態を表す名前（例: `http_requests_in_flight`）

### 2. ラベルの使用

- **適切なカーディナリティ**: ラベルの値が多すぎないようにする
- **一貫性**: 同じ概念には同じラベル名を使用
- **避けるべき**: ユーザーID、リクエストIDなど高カーディナリティの値

### 3. サンプリング

本番環境では、全てのリクエストをトレースするとオーバーヘッドが大きいため、サンプリングを検討：

```rust
// 10%のリクエストのみトレース
if rand::random::<f64>() < 0.1 {
    // トレース処理
}
```

### 4. アラート設定

- **重要度に応じた閾値**: Critical, Warning, Info
- **適切な時間窓**: 短すぎるとノイズ、長すぎると検出遅延
- **アラート疲れの回避**: 重要なアラートのみ設定

## トラブルシューティング

### メトリクスが収集されない

**確認事項**:
1. OpenTelemetryが有効になっているか
2. メトリクスコードが実行されているか
3. エクスポーターが正しく設定されているか

**解決方法**:
```bash
# デバッグログを有効化
RUST_LOG=debug cargo run

# メトリクス収集を確認
./verify-metrics.sh
```

### Prometheusでメトリクスが表示されない

**確認事項**:
1. `/metrics` エンドポイントが実装されているか
2. Prometheusの設定が正しいか
3. ネットワーク接続が正常か

**解決方法**:
```bash
# メトリクスエンドポイントを確認
curl http://localhost:8080/metrics

# Prometheusのターゲット状態を確認
open http://localhost:9090/targets
```

## まとめ

本APIサーバーでは、包括的なメトリクス収集が実装されています：

✅ **実装済み**:
- HTTPメトリクス（リクエスト数、レスポンスタイム、同時実行数）
- データベースメトリクス（クエリ数、実行時間、プール状態）
- 認証メトリクス（認証試行、JWT検証）

📝 **オプション**:
- Prometheus統合（長期保存、高度なクエリ）
- Grafanaダッシュボード（視覚化）
- アラート設定（異常検知）

メトリクスを活用することで、パフォーマンス監視、エラー検出、セキュリティ監視、キャパシティプランニングが可能になります。
