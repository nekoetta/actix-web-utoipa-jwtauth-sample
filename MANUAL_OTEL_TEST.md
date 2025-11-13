# OpenTelemetry 手動テストガイド

このガイドでは、OpenTelemetry統合を手動でステップバイステップでテストする方法を説明します。

## クイックスタート（5分）

### 1. サービスの起動

```bash
# Jaeger と PostgreSQL を起動
docker compose -f docker-compose.otel.yml up -d

# 起動確認（約5秒待つ）
docker ps
```

### 2. データベースのセットアップ

```bash
# マイグレーション実行
diesel migration run
```

### 3. APIサーバーの起動（OpenTelemetry有効）

```bash
# 環境変数を設定してサーバー起動
OTEL_ENABLED=true \
OTEL_ENDPOINT=http://localhost:4317 \
OTEL_SERVICE_NAME=rust-api \
RUST_LOG=info \
cargo run --release
```

### 4. APIコールの実行

別のターミナルで：

```bash
# テスト1: OpenAPI仕様取得（認証不要）
curl http://localhost:8080/api-doc/openapi.json

# テスト2: Swagger UI アクセス（認証不要）
curl http://localhost:8080/swagger-ui/

# テスト3: ユーザー一覧取得（認証必要 - 401エラーになる）
curl http://localhost:8080/api/users/

# テスト4: 顧客カテゴリ一覧（認証必要 - 401エラーになる）
curl http://localhost:8080/api/customers/categories
```

### 5. Jaegerでトレース確認

1. ブラウザで http://localhost:16686 を開く
2. Service ドロップダウンから `rust-api` を選択
3. "Find Traces" ボタンをクリック
4. トレース一覧が表示されることを確認
5. 任意のトレースをクリックして詳細を確認

**確認ポイント**:
- ✅ HTTPリクエストのスパンが表示される
- ✅ http.method, http.target, http.status_code が記録されている
- ✅ リクエストIDが生成されている
- ✅ 実行時間が記録されている

## 詳細テスト

### テスト1: HTTPトレーシングの確認

#### 目的
HTTPリクエストが正しくトレースされることを確認

#### 手順

1. 複数のエンドポイントにアクセス：

```bash
# 成功するリクエスト（200 OK）
curl -v http://localhost:8080/api-doc/openapi.json
curl -v http://localhost:8080/swagger-ui/

# 失敗するリクエスト（401 Unauthorized）
curl -v http://localhost:8080/api/users/
curl -v http://localhost:8080/api/customers/categories

# 存在しないエンドポイント（404 Not Found）
curl -v http://localhost:8080/api/nonexistent
```

2. Jaeger UIで確認：
   - 各リクエストのトレースが表示される
   - HTTPステータスコードが正しく記録されている
   - 200, 401, 404 のトレースがそれぞれ存在する

#### 期待される結果

- ✅ 全てのHTTPリクエストがトレースされる
- ✅ ステータスコードが正しく記録される
- ✅ リクエストパスが記録される
- ✅ HTTPメソッドが記録される

### テスト2: エラートレーシングの確認

#### 目的
エラーが発生した場合にトレースに記録されることを確認

#### 手順

1. 認証エラーを発生させる：

```bash
# 無効なトークンで認証
curl -H "Authorization: Bearer invalid_token" \
     http://localhost:8080/api/users/
```

2. バリデーションエラーを発生させる（認証が必要なので401になる）：

```bash
# 長すぎるカテゴリ名（256文字）
curl -X POST http://localhost:8080/api/customers/categories \
     -H "Content-Type: application/json" \
     -d '{"name":"'$(printf 'a%.0s' {1..256})'"}'
```

3. Jaeger UIで確認：
   - エラーのトレースが表示される
   - エラー情報が記録されている
   - スパンにエラーフラグが立っている

#### 期待される結果

- ✅ エラーがトレースに記録される
- ✅ エラーメッセージが含まれる
- ✅ スパンがエラーとしてマークされる

### テスト3: データベーストレーシングの確認

#### 目的
データベースクエリが正しくトレースされることを確認

#### 前提条件
認証が必要なため、まずテストユーザーを作成する必要があります。

#### 手順（LDAPサーバーがある場合）

1. ログインしてトークンを取得：

```bash
# ログイン
RESPONSE=$(curl -X POST http://localhost:8080/login \
     -H "Content-Type: application/json" \
     -d '{"username":"testuser","password":"testpass"}')

# トークンを抽出（jqを使用）
TOKEN=$(echo $RESPONSE | jq -r '.authorization' | sed 's/Bearer //')

# または手動でレスポンスからトークンをコピー
```

2. 認証付きでAPIコール：

```bash
# ユーザー一覧取得（DBクエリが実行される）
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/users/

# 顧客カテゴリ作成（DBインサートが実行される）
curl -X POST http://localhost:8080/api/customers/categories \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"name":"テストカテゴリ1"}'

# 顧客カテゴリ一覧取得（DBクエリが実行される）
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/customers/categories
```

3. Jaeger UIで確認：
   - データベース操作のスパンが表示される
   - db.operation フィールドが記録されている
   - クエリ実行時間が記録されている

#### 期待される結果

- ✅ データベースクエリがトレースされる
- ✅ 操作種別（insert, select）が記録される
- ✅ クエリ実行時間が記録される
- ✅ HTTPリクエストスパンの子スパンとして表示される

### テスト4: 認証トレーシングの確認

#### 目的
JWT認証処理が正しくトレースされることを確認

#### 手順

1. 有効なトークンで認証：

```bash
# 有効なトークンを使用（上記で取得したトークン）
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/users/
```

2. 無効なトークンで認証：

```bash
# 無効なトークン
curl -H "Authorization: Bearer invalid_token_here" \
     http://localhost:8080/api/users/
```

3. トークンなしで認証：

```bash
# Authorizationヘッダーなし
curl http://localhost:8080/api/users/
```

4. Jaeger UIで確認：
   - JWT検証のスパンが表示される
   - auth.token_valid フィールドが記録されている
   - 認証成功/失敗が区別される

#### 期待される結果

- ✅ JWT検証がトレースされる
- ✅ トークンの有効性が記録される
- ✅ 認証失敗時もトレースされる

### テスト5: パフォーマンス比較

#### 目的
OpenTelemetry有効時と無効時のパフォーマンスを比較

#### 手順

1. OpenTelemetry無効でサーバー起動：

```bash
# 既存のサーバーを停止（Ctrl+C）

# OpenTelemetry無効で起動
OTEL_ENABLED=false \
RUST_LOG=warn \
cargo run --release
```

2. ベンチマーク実行（別のターミナル）：

```bash
# Apache Benchを使用
time ab -n 1000 -c 10 http://localhost:8080/api-doc/openapi.json

# または wrk を使用
wrk -t4 -c100 -d10s http://localhost:8080/api-doc/openapi.json
```

3. 結果を記録（例）：
   - Requests/sec: 5000
   - Latency (avg): 2.0ms

4. サーバーを停止（Ctrl+C）

5. OpenTelemetry有効でサーバー起動：

```bash
OTEL_ENABLED=true \
OTEL_ENDPOINT=http://localhost:4317 \
RUST_LOG=warn \
cargo run --release
```

6. 同じベンチマークを実行：

```bash
time ab -n 1000 -c 10 http://localhost:8080/api-doc/openapi.json
```

7. 結果を記録（例）：
   - Requests/sec: 4800
   - Latency (avg): 2.1ms

8. オーバーヘッドを計算：
   - スループット低下: (5000 - 4800) / 5000 = 4%
   - レイテンシ増加: 2.1 - 2.0 = 0.1ms

#### 期待される結果

- ✅ オーバーヘッドは5%未満
- ✅ レイテンシ増加は1-2ms程度
- ✅ 本番環境で許容可能なパフォーマンス

## トレースの見方

### Jaeger UIの使い方

1. **サービス選択**
   - 左上の "Service" ドロップダウンから `rust-api` を選択

2. **トレース検索**
   - "Find Traces" ボタンをクリック
   - 時間範囲を調整（デフォルトは過去1時間）

3. **トレース一覧**
   - 各トレースの概要が表示される
   - 実行時間、スパン数、エラーの有無

4. **トレース詳細**
   - トレースをクリックして詳細を表示
   - スパンの階層構造（ツリー表示）
   - 各スパンの実行時間（タイムライン表示）

5. **スパン詳細**
   - スパンをクリックして詳細を表示
   - Tags: http.method, http.status_code など
   - Logs: エラーメッセージなど

### トレースの例

```
GET /api/users/ [150ms] - 401 Unauthorized
├─ HTTP Request [150ms]
│  ├─ JWT Validation [10ms]
│  │  └─ Token Decode [2ms] - ERROR: Invalid token
│  └─ Error Response [5ms]
```

```
GET /api-doc/openapi.json [50ms] - 200 OK
├─ HTTP Request [50ms]
│  └─ File Read [45ms]
```

## チェックリスト

### 基本動作確認

- [ ] Jaeger が起動している
- [ ] PostgreSQL が起動している
- [ ] APIサーバーが起動している（OpenTelemetry有効）
- [ ] Jaeger UIにアクセスできる
- [ ] Swagger UIにアクセスできる

### トレース確認

- [ ] HTTPリクエストがトレースされる
- [ ] http.method が記録される
- [ ] http.target が記録される
- [ ] http.status_code が記録される
- [ ] リクエストIDが生成される

### エラートレース確認

- [ ] 401エラーがトレースされる
- [ ] 404エラーがトレースされる
- [ ] エラー情報が記録される

### パフォーマンス確認

- [ ] OpenTelemetry無効時のベンチマーク実行
- [ ] OpenTelemetry有効時のベンチマーク実行
- [ ] オーバーヘッドが5%未満
- [ ] レイテンシ増加が許容範囲内

### ドキュメント確認

- [ ] README.mdにOpenTelemetry設定が記載されている
- [ ] OTEL_TESTING.mdが作成されている
- [ ] docker-compose.otel.ymlが作成されている
- [ ] テストスクリプトが動作する

## トラブルシューティング

### Jaegerにトレースが表示されない

```bash
# 1. Jaegerが起動しているか確認
docker ps | grep jaeger

# 2. APIサーバーのログを確認
RUST_LOG=debug cargo run

# 3. 環境変数を確認
echo $OTEL_ENABLED
echo $OTEL_ENDPOINT

# 4. ネットワーク接続を確認
nc -zv localhost 4317
```

### APIサーバーが起動しない

```bash
# 1. エラーメッセージを確認
RUST_BACKTRACE=1 cargo run

# 2. OpenTelemetry無効で起動してみる
OTEL_ENABLED=false cargo run

# 3. 依存関係を更新
cargo update
cargo build
```

### データベース接続エラー

```bash
# 1. PostgreSQLが起動しているか確認
docker ps | grep postgres

# 2. 接続テスト
psql postgresql://test:test@localhost/test_db -c "SELECT 1"

# 3. マイグレーション実行
diesel migration run
```

## 次のステップ

1. ✅ 基本的なトレーシングが動作することを確認
2. ✅ パフォーマンスオーバーヘッドが許容範囲内であることを確認
3. ✅ エラートレーシングが正しく動作することを確認
4. 📝 本番環境への展開計画を立てる
5. 📝 アラート設定を検討する
6. 📝 ダッシュボードを作成する

## まとめ

このガイドに従って、OpenTelemetry統合の動作を手動で確認できます。

**重要なポイント**:
- OpenTelemetryは環境変数で簡単に有効/無効を切り替えられる
- トレースはJaeger UIで視覚的に確認できる
- パフォーマンスオーバーヘッドは最小限（5%未満）
- エラーも含めて全てのリクエストがトレースされる

問題が発生した場合は、`OTEL_TESTING.md` の詳細なトラブルシューティングセクションを参照してください。
