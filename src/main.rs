use axum::{
    Json, Router,
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// TODO: ArcやRwLockとは
type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

#[tokio::main]
async fn main() {
    // 環境変数の読み込み
    dotenv().expect(".env file not found");

    // デバッグログの初期化
    // 環境変数に基づくログフィルタを設定する
    // TODO: 環境変数の意味を調べる
    // .envではRUST_LOG=debugが定義されているこれによりデバッグログ以上の(debug,info,warn,errorが出力される)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_sandbox=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // メモリ内のデータベースを準備
    // スレッドセーフなメモリ内データベースを構築する。これにより非同期コンテキストでのデータ読み書きが可能になる
    let db = Db::default();

    // ルーティング設定AxumのRouterを使用してエンドポイントを登録
    // それぞれのエンドポイントに対しメソッドを登録
    let app = Router::new()
        .route("/todos", get(todos_index).post(todos_create))
        .route("/todos/:id", patch(todos_update).delete(todos_delete))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10)) // タイムアウトを10秒に設定する
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(db); // TODO:ステートとは

    // TODO: HTTPサーバーを起動
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

// 全TODO項目の一覧をJSON形式で返します
async fn todos_index(State(db): State<Db>) -> impl IntoResponse {
    let todos = db.read().unwrap();
    let todos = todos.values().cloned().collect::<Vec<_>>();
    Json(todos)
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}
// 新しいTODO項目の一覧をJSOＮ形式で返す
async fn todos_create(State(db): State<Db>, Json(input): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: input.text,
        completed: false,
    };

    db.write().unwrap().insert(todo.id, todo.clone());

    (StatusCode::CREATED, Json(todo))
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

async fn todos_update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    db.write().unwrap().insert(todo.id, todo.clone());

    Ok(Json(todo))
}

async fn todos_delete(Path(id): Path<Uuid>, State(db): State<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
