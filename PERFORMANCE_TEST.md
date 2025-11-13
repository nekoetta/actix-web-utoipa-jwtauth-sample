# OpenTelemetryパフォーマンステストガイド

このドキュメントでは、OpenTelemetry有効時と無効時のパフォーマンス比較方法を説明します。

## 目次

1. [テスト目的](#テスト目的)
2. [自動テストスクリプト](#自動テストスクリプト)
3. [手動テスト手順](#手動テスト手順)
4. [ベンチマークツール](#ベンチマークツール)
5. [結果の分析](#結果の分析)
6. [期待される結果](#期待される結果)
7. [最適化のヒント](#最適化のヒント)

## テスト目的

OpenTelemetry統合によるパフォーマンスへの影響を測定し、以下を確認します：

- **スループット**: 1秒あたりのリクエスト処理数
- **レイテンシ**: リクエストの応答時間
- **オーバーヘッド**: OpenTelemetry有効時の性能低下率

### 許容基準

- **オーバーヘッド**: 5%未満
- **レイテンシ増加**: 1-2ms程度
- **スループット低下**: 5%未満

## 自動テストスクリプト

最も簡単な方法は、提供されているベンチマークスクリプトを使用することです。

### 実行方法

```bash
./benchmark-otel.sh
```

### スクリプトの動作

1. ✅ 必要なツールの確認（wrk, ab, hey のいずれか）
2. ✅ PostgreSQL と Jaeger の起動
3. ✅ データベースマイグレーション
4. ✅ アプリケーションのビルド（リリースモード）
5. ✅ OpenTelemetry無効でベンチマーク実行
6. ✅ OpenTelemetry有効でベンチマーク実行
7. ✅ 結果の比較と表示

### 出力例

```
=========================================
Benchmark: otel_disabled
=========================================
ℹ Starting API server (OTEL_ENABLED=false)...
✓ Server is running (PID: 12345)
ℹ Warming up...
ℹ Running benchmark...

Running 30s test @ http://localhost:8080/api-doc/openapi.json
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.00ms    1.50ms   50.00ms   95.00%
    Req/Sec     1.25k   100.00    1.50k    90.00%
  150000 requests in 30.00s, 500.00MB read
Requests/sec:   5000.00
Transfer/sec:     16.67MB

=========================================
Benchmark: otel_enabled
=========================================
ℹ Starting API server (OTEL_ENABLED=true)...
✓ Server is running (PID: 12346)
ℹ Warming up...
ℹ Running benchmark...

Running 30s test @ http://localhost:8080/api-doc/openapi.json
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.10ms    1.55ms   52.00ms   95.00%
    Req/Sec     1.19k   100.00    1.45k    90.00%
  143000 requests in 30.00s, 476.67MB read
Requests/sec:   4766.67
Transfer/sec:     15.89MB

=========================================
Benchmark Results Summary
=========================================

OpenTelemetry DISABLED:
------------------------
Requests/sec:   5000.00
Latency         2.00ms

OpenTelemetry ENABLED:
----------------------
Requests/sec:   4766.67
Latency         2.10ms

ℹ Detailed results saved to:
ℹ   - benchmark_otel_disabled.txt
ℹ   - benchmark_otel_enabled.txt

=========================================
Benchmark Complete
=========================================
✓ Performance comparison completed successfully!
```

### オーバーヘッドの計算

```
スループット低下率 = (5000 - 4766.67) / 5000 × 100% = 4.67%
レイテンシ増加 = 2.10 - 2.00 = 0.10ms
```

## 手動テスト手順

自動スクリプトを使用しない場合の手動テスト手順です。

### 前提条件

1. ベンチマークツールのインストール（いずれか1つ）：

```bash
# wrk（推奨）
sudo apt-get install wrk  # Ubuntu/Debian
brew install wrk          # macOS

# Apache Bench
sudo apt-get install apache2-utils  # Ubuntu/Debian
brew install httpd                  # macOS

# hey
go install github.com/rakyll/hey@latest
```

2. サービスの起動：

```bash
# PostgreSQL と Jaeger を起動
docker compose -f docker-compose.otel.yml up -d

# マイグレーション実行
diesel migration run

# アプリケーションをリリースモードでビルド
cargo build --release
```

### ステップ1: OpenTelemetry無効でのベンチマーク

#### 1.1 サーバー起動

```bash
# OpenTelemetry無効で起動
OTEL_ENABLED=false \
RUST_LOG=warn \
./target/release/rust_api
```

#### 1.2 ベンチマーク実行

別のターミナルで：

```bash
# wrkを使用する場合
wrk -t4 -c100 -d30s --latency http://localhost:8080/api-doc/openapi.json

# Apache Benchを使用する場合
ab -n 10000 -c 100 http://localhost:8080/api-doc/openapi.json

# heyを使用する場合
hey -n 10000 -c 100 http://localhost:8080/api-doc/openapi.json
```

#### 1.3 結果の記録

結果をテキストファイルに保存：

```bash
wrk -t4 -c100 -d30s --latency http://localhost:8080/api-doc/openapi.json > benchmark_otel_disabled.txt
```

重要な指標を記録：
- Requests/sec（スループット）
- Latency Average（平均レイテンシ）
- Latency P95（95パーセンタイル）
- Latency P99（99パーセンタイル）

#### 1.4 サーバー停止

```bash
# Ctrl+C でサーバーを停止
```

### ステップ2: OpenTelemetry有効でのベンチマーク

#### 2.1 サーバー起動

```bash
# OpenTelemetry有効で起動
OTEL_ENABLED=true \
OTEL_ENDPOINT=http://localhost:4317 \
OTEL_SERVICE_NAME=rust-api-benchmark \
RUST_LOG=warn \
./target/release/rust_api
```

#### 2.2 ベンチマーク実行

同じコマンドを実行：

```bash
wrk -t4 -c100 -d30s --latency http://localhost:8080/api-doc/openapi.json > benchmark_otel_enabled.txt
```

#### 2.3 結果の記録

同じ指標を記録し、比較します。

### ステップ3: 結果の比較

#### 3.1 スループットの比較

```bash
# 無効時
grep "Requests/sec:" benchmark_otel_disabled.txt

# 有効時
grep "Requests/sec:" benchmark_otel_enabled.txt
```

#### 3.2 レイテンシの比較

```bash
# 無効時
grep "Latency" benchmark_otel_disabled.txt

# 有効時
grep "Latency" benchmark_otel_enabled.txt
```

#### 3.3 オーバーヘッドの計算

```
オーバーヘッド(%) = (無効時の値 - 有効時の値) / 無効時の値 × 100
```

## ベンチマークツール

### wrk（推奨）

**特徴**:
- 高性能なHTTPベンチマークツール
- Luaスクリプトでカスタマイズ可能
- 詳細なレイテンシ統計

**基本的な使用方法**:

```bash
# 基本
wrk -t4 -c100 -d30s http://localhost:8080/api-doc/openapi.json

# レイテンシ詳細付き
wrk -t4 -c100 -d30s --latency http://localhost:8080/api-doc/openapi.json

# カスタムヘッダー付き
wrk -t4 -c100 -d30s -H "Authorization: Bearer token" http://localhost:8080/api/users/
```

**オプション**:
- `-t`: スレッド数
- `-c`: コネクション数
- `-d`: テスト期間
- `--latency`: レイテンシ統計を表示

**出力例**:

```
Running 30s test @ http://localhost:8080/api-doc/openapi.json
  4 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.00ms    1.50ms   50.00ms   95.00%
    Req/Sec     1.25k   100.00    1.50k    90.00%
  Latency Distribution
     50%    1.80ms
     75%    2.20ms
     90%    2.80ms
     99%    5.00ms
  150000 requests in 30.00s, 500.00MB read
Requests/sec:   5000.00
Transfer/sec:     16.67MB
```

### Apache Bench (ab)

**特徴**:
- シンプルで使いやすい
- 広く使われている標準ツール
- 基本的な統計情報

**基本的な使用方法**:

```bash
# 基本
ab -n 10000 -c 100 http://localhost:8080/api-doc/openapi.json

# 詳細出力
ab -n 10000 -c 100 -v 2 http://localhost:8080/api-doc/openapi.json

# カスタムヘッダー付き
ab -n 10000 -c 100 -H "Authorization: Bearer token" http://localhost:8080/api/users/
```

**オプション**:
- `-n`: リクエスト総数
- `-c`: 同時接続数
- `-v`: 詳細レベル（0-4）

**出力例**:

```
Benchmarking localhost (be patient)
Completed 1000 requests
...
Completed 10000 requests
Finished 10000 requests

Server Software:        
Server Hostname:        localhost
Server Port:            8080

Document Path:          /api-doc/openapi.json
Document Length:        3456 bytes

Concurrency Level:      100
Time taken for tests:   2.000 seconds
Complete requests:      10000
Failed requests:        0
Total transferred:      35000000 bytes
HTML transferred:       34560000 bytes
Requests per second:    5000.00 [#/sec] (mean)
Time per request:       20.000 [ms] (mean)
Time per request:       0.200 [ms] (mean, across all concurrent requests)
Transfer rate:          17500.00 [Kbytes/sec] received

Connection Times (ms)
              min  mean[+/-sd] median   max
Connect:        0    1   0.5      1       5
Processing:     1   19   5.0     18      50
Waiting:        1   18   5.0     17      49
Total:          2   20   5.0     19      51

Percentage of the requests served within a certain time (ms)
  50%     19
  66%     21
  75%     23
  80%     24
  90%     27
  95%     30
  98%     35
  99%     40
 100%     51 (longest request)
```

### hey

**特徴**:
- Go言語で書かれた高速ツール
- シンプルなインターフェース
- カラフルな出力

**基本的な使用方法**:

```bash
# 基本
hey -n 10000 -c 100 http://localhost:8080/api-doc/openapi.json

# カスタムヘッダー付き
hey -n 10000 -c 100 -H "Authorization: Bearer token" http://localhost:8080/api/users/

# POSTリクエスト
hey -n 1000 -c 10 -m POST -H "Content-Type: application/json" \
  -d '{"name":"test"}' http://localhost:8080/api/customers/categories
```

**オプション**:
- `-n`: リクエスト総数
- `-c`: 同時接続数
- `-m`: HTTPメソッド
- `-H`: カスタムヘッダー
- `-d`: リクエストボディ

## 結果の分析

### 重要な指標

#### 1. スループット（Requests/sec）

**意味**: 1秒あたりに処理できるリクエスト数

**評価**:
- 高いほど良い
- OpenTelemetry有効時の低下が5%未満なら許容範囲

**計算例**:
```
無効時: 5000 req/sec
有効時: 4800 req/sec
低下率: (5000 - 4800) / 5000 = 4% ✅ 許容範囲内
```

#### 2. 平均レイテンシ（Average Latency）

**意味**: リクエストの平均応答時間

**評価**:
- 低いほど良い
- OpenTelemetry有効時の増加が1-2ms程度なら許容範囲

**計算例**:
```
無効時: 2.0ms
有効時: 2.1ms
増加: 2.1 - 2.0 = 0.1ms ✅ 許容範囲内
```

#### 3. P95/P99レイテンシ

**意味**: 95%/99%のリクエストがこの時間以内に完了

**評価**:
- ユーザー体験に直結する重要な指標
- P99が大きく増加していないか確認

**例**:
```
P95: 無効時 5.0ms → 有効時 5.2ms (増加 0.2ms) ✅
P99: 無効時 10.0ms → 有効時 10.5ms (増加 0.5ms) ✅
```

### 結果の記録テンプレート

```markdown
## パフォーマンステスト結果

### テスト環境
- **日時**: 2024-01-15 10:00:00
- **CPU**: Intel Core i7-9700K @ 3.60GHz (8 cores)
- **メモリ**: 16GB DDR4
- **OS**: Ubuntu 22.04 LTS
- **Rust**: 1.75.0
- **ベンチマークツール**: wrk 4.2.0

### テスト条件
- **スレッド数**: 4
- **同時接続数**: 100
- **テスト期間**: 30秒
- **エンドポイント**: GET /api-doc/openapi.json

### OpenTelemetry無効時
- **Requests/sec**: 5000.00
- **Latency (avg)**: 2.00ms
- **Latency (p50)**: 1.80ms
- **Latency (p95)**: 5.00ms
- **Latency (p99)**: 10.00ms
- **Total requests**: 150,000

### OpenTelemetry有効時
- **Requests/sec**: 4800.00
- **Latency (avg)**: 2.10ms
- **Latency (p50)**: 1.90ms
- **Latency (p95)**: 5.20ms
- **Latency (p99)**: 10.50ms
- **Total requests**: 144,000

### オーバーヘッド
- **スループット低下**: 4.0% ✅
- **レイテンシ増加 (avg)**: 0.10ms ✅
- **レイテンシ増加 (p95)**: 0.20ms ✅
- **レイテンシ増加 (p99)**: 0.50ms ✅

### 結論
OpenTelemetry有効時のオーバーヘッドは4.0%で、許容範囲内（5%未満）です。
レイテンシの増加も最小限（0.1-0.5ms）であり、本番環境での使用に問題ありません。
```

## 期待される結果

### 理想的な結果

- **スループット低下**: 3-5%
- **平均レイテンシ増加**: 0.1-0.5ms
- **P95レイテンシ増加**: 0.2-1.0ms
- **P99レイテンシ増加**: 0.5-2.0ms

### 許容範囲

- **スループット低下**: 10%未満
- **平均レイテンシ増加**: 2ms未満
- **P95レイテンシ増加**: 5ms未満
- **P99レイテンシ増加**: 10ms未満

### 要改善

以下の場合は最適化が必要：

- **スループット低下**: 10%以上
- **平均レイテンシ増加**: 2ms以上
- **P95レイテンシ増加**: 5ms以上
- **P99レイテンシ増加**: 10ms以上

## 最適化のヒント

### 1. サンプリングレートの調整

全てのリクエストをトレースするとオーバーヘッドが大きいため、サンプリングを検討：

```rust
// 10%のリクエストのみトレース
use rand::Rng;

if rand::thread_rng().gen::<f64>() < 0.1 {
    // トレース処理
}
```

### 2. バッチエクスポートの最適化

トレースデータをバッチでエクスポートすることで、ネットワークオーバーヘッドを削減：

```rust
use opentelemetry::sdk::trace::BatchConfig;

let batch_config = BatchConfig::default()
    .with_max_queue_size(2048)
    .with_max_export_batch_size(512)
    .with_scheduled_delay(std::time::Duration::from_secs(5));
```

### 3. ログレベルの調整

本番環境では、ログレベルを `warn` または `error` に設定：

```bash
RUST_LOG=warn cargo run --release
```

### 4. 非同期エクスポート

トレースデータを非同期でエクスポートすることで、リクエスト処理をブロックしない：

```rust
// 既に実装済み（Tokioランタイムを使用）
```

### 5. 不要なスパンの削除

詳細すぎるトレーシングは避け、重要な操作のみトレース：

```rust
// ❌ 避けるべき
#[instrument]
fn small_helper_function() { }

// ✅ 推奨
#[instrument(skip_all)]  // 引数をスキップ
fn important_operation() { }
```

## トラブルシューティング

### パフォーマンスが著しく低下する

**原因**:
- ログレベルが `debug` になっている
- 全てのリクエストをトレースしている
- ネットワーク遅延が大きい

**解決方法**:
```bash
# ログレベルを調整
RUST_LOG=warn cargo run --release

# サンプリングレートを下げる（コード修正）
# ネットワーク接続を確認
```

### ベンチマークツールがエラーを返す

**原因**:
- サーバーが起動していない
- ポートが間違っている
- 同時接続数が多すぎる

**解決方法**:
```bash
# サーバーの起動確認
curl http://localhost:8080/swagger-ui/

# 同時接続数を減らす
wrk -t2 -c50 -d10s http://localhost:8080/api-doc/openapi.json
```

### 結果が不安定

**原因**:
- ウォームアップ不足
- バックグラウンドプロセスの影響
- リソース不足

**解決方法**:
```bash
# ウォームアップを追加
for i in {1..100}; do curl -s http://localhost:8080/api-doc/openapi.json > /dev/null; done

# バックグラウンドプロセスを停止
# リソース使用状況を確認
htop
```

## まとめ

OpenTelemetryのパフォーマンステストは、以下の手順で実行できます：

1. ✅ 自動スクリプトを実行: `./benchmark-otel.sh`
2. ✅ 結果を確認: `benchmark_*.txt`
3. ✅ オーバーヘッドを計算
4. ✅ 許容範囲内か判断

**期待される結果**:
- スループット低下: 5%未満
- レイテンシ増加: 1-2ms程度

**最適化のポイント**:
- サンプリングレートの調整
- ログレベルの設定
- バッチエクスポートの最適化

問題が発生した場合は、トラブルシューティングセクションを参照してください。
