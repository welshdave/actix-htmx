use sqlx::FromRow;
use std::fmt;
use uuid::Uuid;

#[derive(PartialEq, Copy, Clone, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum Status {
    Pending,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "Pending"),
            Status::Done => write!(f, "Done"),
        }
    }
}

#[derive(FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub name: String,
    pub status: Status,
}

pub struct Todos;

impl Todos {
    pub async fn get_todos(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<Todo>, sqlx::Error> {
        let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
            .fetch_all(pool)
            .await?;

        Ok(todos)
    }

    pub async fn add_todo(pool: &sqlx::Pool<sqlx::Sqlite>, name: &str) -> Result<(), sqlx::Error> {
        let new_id = Uuid::new_v4();

        if name.trim().is_empty() {
            return Ok(());
        }

        sqlx::query!(
            r#"
            INSERT INTO todos (id, name, status)
            VALUES (?1, ?2, ?3);
            "#,
            new_id,
            name,
            Status::Pending
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_todo(pool: &sqlx::Pool<sqlx::Sqlite>, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM todos WHERE id = ?1
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn update_todo(
        pool: &sqlx::Pool<sqlx::Sqlite>,
        status: Status,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE todos SET Status = ?1 WHERE id = ?2
            "#,
            status,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
