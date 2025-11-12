use axum::{
    extract::{State, Path},
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use chrono::{Utc, DateTime};
use tower_http::cors::{CorsLayer, Any};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Clone)]
struct TodoItem {
    id: Uuid,
    title: String,
    completed: bool,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct CreateTodoItem {
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct UpdateTodoItem {
    title: Option<String>,
    completed: Option<bool>,
}

#[derive(Clone)]
struct AppState {
    todo_list: Arc<Mutex<Vec<TodoItem>>>,
}

async fn get_todos(State(data): State<AppState>) -> impl IntoResponse {
    let todos = data.todo_list.lock().unwrap();
    (StatusCode::OK, Json(todos.clone()))
}

async fn add_todos(
    State(data): State<AppState>,
    Json(item): Json<CreateTodoItem>,
) -> impl IntoResponse {
    let mut todos = data.todo_list.lock().unwrap();
    let new_todo = TodoItem {
        id: Uuid::new_v4(),
        title: item.title,
        completed: item.completed,
        created_at: Utc::now(),
    };
    todos.push(new_todo);
    (StatusCode::OK, Json(todos.clone()))
}

async fn update_todos(
    State(data): State<AppState>,
    Path(id): Path<Uuid>,
    Json(item): Json<UpdateTodoItem>,
) -> impl IntoResponse {
    let mut todos = data.todo_list.lock().unwrap();

    if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
        if let Some(title) = item.title {
            todo.title = title;
        }
        if let Some(completed) = item.completed {
            todo.completed = completed;
        }
        (StatusCode::OK, Json(todos.clone())).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Todo not found").into_response()
    }
}

async fn delete_todos(State(data): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut todos = data.todo_list.lock().unwrap();

    if todos.iter().any(|t| t.id == id) {
        todos.retain(|t| t.id != id);
        (StatusCode::OK, Json(todos.clone())).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Todo not found").into_response()
    }
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        todo_list: Arc::new(Mutex::new(Vec::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/todos", get(get_todos).post(add_todos))
        .route("/todos/:id", put(update_todos).delete(delete_todos))
        .with_state(app_state)
        .layer(cors);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("🚀 Server running on http://127.0.0.1:8080");

    axum::serve(listener, app).await.unwrap();
}
