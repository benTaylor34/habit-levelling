use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

#[derive(Serialize, Deserialize)]
pub struct Habit {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub frequency: String,
    pub active: bool,
}

#[derive(Deserialize)]
pub struct NewHabit {
    pub name: String,
    pub description: Option<String>,
    pub frequency: String,
}

pub async fn create_habit(
    pool: State<'_, SqlitePool>,
    habit: NewHabit,
) -> Result<Habit, String> {
    sqlx::query_as!(
        Habit,
        "INSERT INTO habits (name, description, frequency, active)
        VALUES (?, ?, ?, true)
        RETURNING id, name, description, frequency, active",
        habit.name, habit.description, habit.frequency
    )
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())
}