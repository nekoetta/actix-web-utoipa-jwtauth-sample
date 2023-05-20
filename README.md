# actix-web api server

- [actix-web api server](#actix-web-api-server)
  - [module構造](#module構造)
  - [環境変数](#環境変数)
  - [起動方法(Docker)](#起動方法docker)
  - [起動方法(Dockerを使用しない)](#起動方法dockerを使用しない)
  - [開発方法](#開発方法)
  - [テスト実行方法](#テスト実行方法)
  - [openapi specification生成方法](#openapi-specification生成方法)
  - [TODO](#todo)

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

- DATABASE_URL
  - 例: DATABASE_URL=postgres://postgres:P%40ssw0rd@localhost/development
- TEST_DATABASE_URL
  - 例: TEST_DATABASE_URL=postgres://postgres:P%40ssw0rd@localhost:5433/test
- JWT_SECRET
  - 例: JWT_SECRET="18 A6 77 73 7F 72 44 6C 26 84 0B 19 75 E0 07 FA 73 A4 A8 82 21 C7 99 AC 0D C6 A5 FE D0 E4 E0 E6"
- LDAP_URI=ldap://ad.example.com
- LDAP_UID_COLUMN=cn
- LDAP_FILTER="(objectCategory=CN=Person*)"
- LDAP_USER_DN="cn=users,dc=example,dc=com"
- LDAP_GUARD_FILTER="(objectCategory=CN=Group*)"

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

1. 以下のコマンドを実行します  
   `RUST_BACKTRACE=1 RUST_LOG=debug cargo test`

## openapi specification生成方法

1. 以下のコマンドを実行します。  
    ```cargo run --bin generate_openapi_schema```
2. openapi_schema.json が出力されます

## TODO

  tag, context_pathに定数を指定する。 <https://github.com/juhaku/utoipa/issues/518>
