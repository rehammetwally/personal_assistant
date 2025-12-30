use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use sqlx::{Sqlite, SqlitePool};
use uuid::Uuid;

use crate::auth::{create_jwt, hash_password, verify_password, AuthenticatedUser};
use crate::groq::GroqClient;
use crate::models::*;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub groq: Option<GroqClient>,
}

impl axum::extract::FromRef<AppState> for sqlx::SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Auth routes
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/me", get(get_me))
        
        // Task routes
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/:id", patch(update_task).delete(delete_task))
        
        // Expense routes
        .route("/api/expenses", get(list_expenses).post(create_expense))
        .route("/api/expenses/:id", delete(delete_expense))
        .route("/api/expenses/summary", get(get_expense_summary))
        
        // AI routes
        .route("/api/ai/suggest", post(ai_suggest))
        .route("/api/ai/chat", post(ai_chat_handler))
        .with_state(state)
}

// --- Auth Handlers ---

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let hashed = hash_password(&payload.password);

    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&payload.email)
        .bind(&hashed)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("User registration failed: {}", e)))?;

    let user = User {
        id,
        email: payload.email,
        password_hash: "".to_string(), // Don't return hash
        created_at: chrono::Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(user)))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = sqlx::query_as::<Sqlite, User>("SELECT * FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
    }

    let token = create_jwt(&user.id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(AuthResponse { token, user }))
}

async fn get_me(AuthenticatedUser(user): AuthenticatedUser) -> impl IntoResponse {
    Json(user)
}

// --- Task Handlers ---

async fn list_tasks(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let tasks = sqlx::query_as::<Sqlite, Task>("SELECT * FROM tasks WHERE user_id = ? ORDER BY created_at DESC")
        .bind(user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(tasks))
}

async fn create_task(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    
    sqlx::query("INSERT INTO tasks (id, user_id, title) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&user.id)
        .bind(&payload.title)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let task = sqlx::query_as::<Sqlite, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(task)))
}

async fn update_task(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if let Some(completed) = payload.completed {
        sqlx::query("UPDATE tasks SET completed = ? WHERE id = ? AND user_id = ?")
            .bind(completed)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    if let Some(title) = payload.title {
        sqlx::query("UPDATE tasks SET title = ? WHERE id = ? AND user_id = ?")
            .bind(title)
            .bind(&id)
            .bind(&user.id)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let task = sqlx::query_as::<Sqlite, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(task))
}

async fn delete_task(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM tasks WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// --- Expense Handlers ---

async fn list_expenses(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let expenses = sqlx::query_as::<Sqlite, Expense>("SELECT * FROM expenses WHERE user_id = ? ORDER BY created_at DESC")
        .bind(user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(expenses))
}

async fn create_expense(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateExpenseRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO expenses (id, user_id, category, amount) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(&user.id)
        .bind(&payload.category)
        .bind(payload.amount)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let expense = sqlx::query_as::<Sqlite, Expense>("SELECT * FROM expenses WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(expense)))
}

async fn delete_expense(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    sqlx::query("DELETE FROM expenses WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_expense_summary(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let total: (f64,) = sqlx::query_as::<Sqlite, (f64,)>("SELECT COALESCE(SUM(amount), 0.0) FROM expenses WHERE user_id = ?")
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let categories: Vec<(String, f64)> = sqlx::query_as::<Sqlite, (String, f64)>("SELECT category, SUM(amount) FROM expenses WHERE user_id = ? GROUP BY category ORDER BY SUM(amount) DESC")
        .bind(&user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(BudgetSummary {
        total_spending: total.0,
        categories,
    }))
}

// --- AI Handlers ---

async fn ai_suggest(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = state.groq.ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "AI features disabled".to_string()))?;
    
    // Fetch data for context
    let tasks = sqlx::query_as::<Sqlite, Task>("SELECT * FROM tasks WHERE user_id = ? AND completed = 0")
        .bind(&user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total: (f64,) = sqlx::query_as::<Sqlite, (f64,)>("SELECT COALESCE(SUM(amount), 0.0) FROM expenses WHERE user_id = ?")
        .bind(&user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let task_list = tasks.iter().map(|t| t.title.clone()).collect::<Vec<_>>().join(", ");
    
    let prompt = format!(
        "User context:
Tasks pending: {}
Total spending: ${:.2}

Provide a brief, motivating suggestion for what they should do next.",
        if task_list.is_empty() { "None" } else { &task_list },
        total.0
    );

    let response = client.quick_chat(&prompt).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({ "suggestion": response })))
}

async fn ai_chat_handler(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let client = state.groq.ok_or_else(|| (StatusCode::SERVICE_UNAVAILABLE, "AI features disabled".to_string()))?;

    // Get recent chat history
    let history = sqlx::query_as::<Sqlite, ChatMessage>("SELECT * FROM chat_messages WHERE user_id = ? ORDER BY created_at DESC LIMIT 10")
        .bind(&user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut messages: Vec<crate::groq::Message> = Vec::new();
    messages.push(crate::groq::Message::system("You are a helpful SaaS personal assistant."));
    
    for msg in history.into_iter().rev() {
        messages.push(crate::groq::Message {
            role: msg.role,
            content: msg.content,
        });
    }
    
    messages.push(crate::groq::Message::user(&payload.message));

    let response = client.chat(messages).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Save message pair
    let user_msg_id = Uuid::new_v4().to_string();
    let assistant_msg_id = Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO chat_messages (id, user_id, role, content) VALUES (?, ?, 'user', ?)")
        .bind(user_msg_id)
        .bind(&user.id)
        .bind(&payload.message)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("INSERT INTO chat_messages (id, user_id, role, content) VALUES (?, ?, 'assistant', ?)")
        .bind(assistant_msg_id)
        .bind(&user.id)
        .bind(&response)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "response": response })))
}
