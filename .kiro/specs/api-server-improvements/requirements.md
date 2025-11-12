# Requirements Document

## Introduction

このドキュメントは、Rust APIサーバー(Actix-web + Diesel + JWT + LDAP)の包括的な仕様定義と改善要件を記述します。既存システムの機能仕様を明確化し、Obsidianの実装観点に基づいたコード品質の検証、OpenTelemetry統合による可観測性の向上、およびドキュメントの改善を行います。

## Glossary

- **API Server**: Actix-webフレームワークを使用したREST APIサーバー
- **Authentication System**: LDAP統合とJWTトークンベースの認証システム
- **User**: システムにログインするユーザー。LDAPディレクトリから情報を取得し、ローカルDBに保存される
- **Customer Category**: 顧客分類を管理するエンティティ
- **OpenTelemetry**: 分散トレーシング、メトリクス、ログを統合的に扱うための可観測性フレームワーク
- **Telemetry System**: トレース、メトリクス、ログを収集・エクスポートするシステム
- **Specification Document**: 現在のシステムの仕様を記述したドキュメント
- **Implementation Guidelines**: Obsidianに記載された実装観点のガイドライン
- **Validation System**: validatorクレートを使用したデータ検証システム

## Requirements

### Requirement 1: ユーザー認証機能

**User Story:** ユーザーとして、LDAPアカウントでAPIサーバーにログインしたい。これにより、既存の組織アカウントを使用してシステムにアクセスできる。

#### Acceptance Criteria

1. WHEN ユーザーがログイン情報を送信する時、THE Authentication System SHALL LDAPサーバーに対して認証を実行する
2. WHEN LDAP認証が成功した時、THE Authentication System SHALL ユーザー情報(employeeNumber、氏名、メールアドレス)をLDAPから取得する
3. WHEN 認証されたユーザーがローカルDBに存在しない時、THE Authentication System SHALL ユーザー情報をusersテーブルに登録する
4. WHEN LDAP認証が成功した時、THE Authentication System SHALL JWTトークンを生成してAuthorizationヘッダーで返却する
5. WHERE ユーザーがPartnerグループに所属している場合、THE Authentication System SHALL ログインを拒否する

### Requirement 2: JWT認証ミドルウェア

**User Story:** 開発者として、保護されたAPIエンドポイントへのアクセスを制御したい。これにより、認証されたユーザーのみがリソースにアクセスできる。

#### Acceptance Criteria

1. WHEN 保護されたエンドポイントにリクエストが送信される時、THE API Server SHALL Authorizationヘッダーからトークンを抽出する
2. WHEN JWTトークンを検証する時、THE API Server SHALL 設定されたシークレットキーを使用してHS256アルゴリズムで検証する
3. WHEN トークンが無効または期限切れの場合、THE API Server SHALL 401 Unauthorizedレスポンスを返す
4. WHEN トークンが有効な場合、THE API Server SHALL トークンからユーザー情報を抽出してリクエストコンテキストに設定する
5. THE API Server SHALL トークンの有効期限を7日間に設定する

### Requirement 3: ユーザー管理API

**User Story:** 管理者として、システムに登録されているユーザー一覧を取得したい。これにより、ユーザー管理を行うことができる。

#### Acceptance Criteria

1. THE API Server SHALL GET /api/users/ エンドポイントでユーザー一覧を返却する
2. WHEN ユーザー一覧を取得する時、THE API Server SHALL JWT認証を要求する
3. THE API Server SHALL ユーザー情報(id、login_id、employee_number、first_name、last_name、email、gecos)をJSON形式で返却する
4. WHEN データベースエラーが発生した時、THE API Server SHALL 500 Internal Server Errorレスポンスを返す
5. THE API Server SHALL 現在のユーザー情報をリクエストコンテキストから取得可能にする

### Requirement 4: 顧客カテゴリ管理API

**User Story:** ユーザーとして、顧客分類を作成・更新・削除したい。これにより、顧客を分類して管理できる。

#### Acceptance Criteria

1. THE API Server SHALL POST /api/customers/categories エンドポイントで新規カテゴリを作成する
2. THE API Server SHALL GET /api/customers/categories エンドポイントでカテゴリ一覧をID昇順で返却する
3. THE API Server SHALL GET /api/customers/categories/{id} エンドポイントで特定カテゴリの詳細を返却する
4. THE API Server SHALL PUT /api/customers/categories/{id}/edit エンドポイントでカテゴリを更新する
5. THE API Server SHALL DELETE /api/customers/categories/{id}/delete エンドポイントでカテゴリを削除する

### Requirement 5: バリデーション機能

**User Story:** 開発者として、不正なデータがデータベースに保存されることを防ぎたい。これにより、データ整合性を保証できる。

#### Acceptance Criteria

1. WHEN カテゴリ名が255文字を超える時、THE Validation System SHALL バリデーションエラーを返す
2. WHEN バリデーションエラーが発生した時、THE API Server SHALL 400 Bad Requestレスポンスを返す
3. THE Validation System SHALL エラーメッセージを日本語で返却する
4. THE Validation System SHALL フィールド名とエラー詳細を含むJSONレスポンスを返す
5. THE Validation System SHALL データベース挿入前にバリデーションを実行する

### Requirement 6: OpenAPI仕様生成

**User Story:** 開発者として、APIの仕様書を自動生成したい。これにより、ドキュメントとコードの同期を保つことができる。

#### Acceptance Criteria

1. THE API Server SHALL Swagger UIを /swagger-ui/ エンドポイントで提供する
2. THE API Server SHALL OpenAPI仕様を /api-doc/openapi.json エンドポイントで提供する
3. THE API Server SHALL utoipaマクロを使用してエンドポイント定義から仕様を生成する
4. THE API Server SHALL Bearer認証スキームをOpenAPI仕様に含める
5. THE API Server SHALL コマンドラインから openapi_schema.json ファイルを生成可能にする

### Requirement 7: データベース管理

**User Story:** 開発者として、データベーススキーマをバージョン管理したい。これにより、環境間でスキーマを一貫して管理できる。

#### Acceptance Criteria

1. THE API Server SHALL Diesel ORMを使用してデータベース操作を実行する
2. THE API Server SHALL マイグレーションファイルでスキーマ変更を管理する
3. THE API Server SHALL PostgreSQLデータベースに接続する
4. THE API Server SHALL コネクションプールを使用してデータベース接続を管理する
5. WHERE テスト実行時、THE API Server SHALL テスト用データベースを使用してマイグレーションを自動実行する

### Requirement 8: エラーハンドリング

**User Story:** 開発者として、統一されたエラーレスポンスを返したい。これにより、クライアント側で一貫したエラー処理を実装できる。

#### Acceptance Criteria

1. THE API Server SHALL ServiceErrorエンumでエラー種別を管理する
2. THE API Server SHALL InternalServerErrorとValidationErrorを区別する
3. WHEN ValidationErrorが発生した時、THE API Server SHALL フィールドエラーの詳細をJSONで返す
4. WHEN InternalServerErrorが発生した時、THE API Server SHALL 汎用エラーメッセージを返す
5. THE API Server SHALL Actix-webのResponseErrorトレイトを実装してエラーレスポンスを生成する

### Requirement 9: CORS設定

**User Story:** フロントエンド開発者として、異なるオリジンからAPIにアクセスしたい。これにより、フロントエンドとバックエンドを分離して開発できる。

#### Acceptance Criteria

1. THE API Server SHALL 環境変数CLIENT_HOSTで指定されたオリジンからのリクエストを許可する
2. THE API Server SHALL GET、POST、PUT、DELETEメソッドを許可する
3. THE API Server SHALL Content-TypeとAuthorizationヘッダーを許可する
4. THE API Server SHALL Authorizationヘッダーをレスポンスで公開する
5. WHERE CLIENT_HOSTが未設定の場合、THE API Server SHALL デフォルトで http://localhost:3000 を許可する

### Requirement 10: テスト機能

**User Story:** 開発者として、自動テストを実行してコードの品質を保証したい。これにより、リグレッションを防止できる。

#### Acceptance Criteria

1. THE API Server SHALL 統合テストをtestsディレクトリに配置する
2. THE API Server SHALL ユニットテストをモジュール内に配置する
3. WHEN テストを実行する時、THE API Server SHALL テスト用データベースを使用する
4. THE API Server SHALL test_transactionを使用してテストデータを自動ロールバックする
5. THE API Server SHALL バリデーションエラーのテストケースを含む

### Requirement 11: コード品質検証

**User Story:** 開発者として、現在のコードベースがObsidianの実装観点に準拠しているか確認したい。これにより、コード品質を保証し、将来のメンテナンス性を向上させることができる。

#### Acceptance Criteria

1. WHEN コードレビューを実施する時、THE Specification Document SHALL 美しいコード観点(関数型思考、命名規則、可視性制御)の準拠状況を記載する
2. WHEN コードレビューを実施する時、THE Specification Document SHALL セキュリティ観点(CSRF、prepared statement、injection対策)の準拠状況を記載する
3. WHEN コードレビューを実施する時、THE Specification Document SHALL エラー設計観点(エラー種別管理、ログ設計)の準拠状況を記載する
4. WHEN コードレビューを実施する時、THE Specification Document SHALL データベース設計観点(トランザクション、インデックス)の準拠状況を記載する
5. WHEN 改善が必要な項目を特定した時、THE Specification Document SHALL 具体的な改善提案を含める

### Requirement 12: OpenTelemetry統合

**User Story:** 運用担当者として、APIサーバーの動作状況をリアルタイムで監視したい。これにより、パフォーマンス問題や障害を早期に検知できる。

#### Acceptance Criteria

1. WHEN HTTPリクエストを受信した時、THE Telemetry System SHALL リクエストのトレース情報を記録する
2. WHEN データベースクエリを実行する時、THE Telemetry System SHALL クエリの実行時間とトレース情報を記録する
3. WHEN エラーが発生した時、THE Telemetry System SHALL エラーの詳細情報とスタックトレースを記録する
4. WHERE OpenTelemetryが有効化されている場合、THE API Server SHALL トレースデータをOTLP形式でエクスポートする
5. WHERE OpenTelemetryが有効化されている場合、THE API Server SHALL メトリクスデータ(リクエスト数、レスポンス時間、エラー率)を収集する

### Requirement 13: OpenTelemetry設定管理

**User Story:** 開発者として、OpenTelemetryを簡単に有効化/無効化したい。これにより、開発環境と本番環境で異なる設定を使用できる。

#### Acceptance Criteria

1. THE API Server SHALL 環境変数でOpenTelemetryの有効/無効を制御する
2. WHERE OpenTelemetryが無効の場合、THE API Server SHALL テレメトリのオーバーヘッドなしで動作する
3. THE API Server SHALL OpenTelemetryエクスポーターのエンドポイントを環境変数で設定可能にする
4. THE API Server SHALL サービス名とバージョンを環境変数で設定可能にする
5. WHERE 設定が不正な場合、THE API Server SHALL 起動時に明確なエラーメッセージを表示する

### Requirement 14: OpenTelemetry実装

**User Story:** 開発者として、既存のコードに最小限の変更でOpenTelemetryを統合したい。これにより、既存機能への影響を最小化できる。

#### Acceptance Criteria

1. THE API Server SHALL Actix-webミドルウェアとしてOpenTelemetryトレーシングを実装する
2. THE API Server SHALL 既存のログ出力を維持しながらOpenTelemetryログを追加する
3. THE API Server SHALL 既存のエラーハンドリングロジックを変更せずにトレース情報を追加する
4. THE API Server SHALL Dieselのクエリ実行にトレーシングを追加する
5. THE API Server SHALL JWT認証処理にトレーシングを追加する

### Requirement 15: ドキュメント改善

**User Story:** 新規参加者として、プロジェクトのセットアップ方法と仕様を素早く理解したい。これにより、開発に迅速に参加できる。

#### Acceptance Criteria

1. THE Specification Document SHALL システムアーキテクチャの概要を含む
2. THE Specification Document SHALL 認証フロー(LDAP + JWT)の詳細を含む
3. THE Specification Document SHALL データモデルとリレーションシップを含む
4. THE Specification Document SHALL APIエンドポイントの一覧と説明を含む
5. THE README Document SHALL OpenTelemetry設定方法を含む
