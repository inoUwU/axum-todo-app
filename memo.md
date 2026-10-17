# Memo

## ライブラリについて

### Axum

型安全ルーティングやリクエストのバリエーション、レスポンスのシリアライズ等の機能を提供する。　　

### serde

データ変換を行うためのライブラリとしてデファクト。　　
JSONやXML等をRustのデータ構造に簡単にデシリアライズ・シリアライズすることができる。
マクロでシリアライズとデシリアライズを自動実装できる。

### Tokio

非同期ランタイム。
Rustは非同期部分の環境毎に切り替えられるようにランタイムで切り出せるようになっている。
Tokioはネットワークサービスや非同期I/O操作、タイマー操作、マルチスレッド処理などの機能を提供している。

## ArcやRwLockとは

### Arc

[Arcの仕組みを理解する-Rustの並行性とメモリ安全性の両立](https://zenn.dev/fast/articles/989065f459b99b)

### 仮想DB

下記コードでは`Arc<RwLock<HaskMap<Uuid,Todo>>>`を使用してメモリ内に仮想的なデータベースを作成している。
ArcとRwLockを組み合わせて複数スレッドから安全にアクセスし読み書きが行えるようにしてある

```rust
use std::{
collections::HashMap,
sync::{Arc, RwLock}
};
use serde::Serialize;
use uuid::Uuid;

#[tokio::main]
async fn main() {
let db = Db::default();
}

type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
id: Uuid,
text: String,
completed: bool,
}
```
