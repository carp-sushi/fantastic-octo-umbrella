use crate::v1::entity::{Status, Story, Task};
use crate::{Error, Result};

use futures_util::TryStreamExt;
use sqlx::postgres::{PgPool, PgRow};
use sqlx::{FromRow, Row};
use std::sync::Arc;
use uuid::Uuid;

pub struct Repo {
    db: Arc<PgPool>,
}

impl Repo {
    /// Constructor
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    /// Get a ref to the connection pool.
    pub fn db_ref(&self) -> &PgPool {
        return self.db.as_ref();
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::InternalError {
            message: err.to_string(),
        }
    }
}

impl FromRow<'_, PgRow> for Story {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            story_id: row.try_get("id")?,
            name: row.try_get("name")?,
            owner: row.try_get("owner")?,
        })
    }
}

impl FromRow<'_, PgRow> for Task {
    fn from_row(row: &PgRow) -> std::result::Result<Self, sqlx::Error> {
        // Extract column values
        let task_id = row.try_get("id")?;
        let story_id = row.try_get("story_id")?;
        let name = row.try_get("name")?;
        let status: String = row.try_get("status")?;

        // Convert to enum type
        let status = Status::try_from(status)
            .map_err(|message| sqlx::Error::Decode(Box::new(Error::InternalError { message })))?;

        // Task
        Ok(Self {
            task_id,
            story_id,
            name,
            status,
        })
    }
}

impl Repo {
    /// Insert a new story
    pub async fn insert_story(&self, name: String, owner: String) -> Result<Story> {
        log::debug!("Repo::insert_story: {}, {}", &name, &owner);

        let sql = r#"
            INSERT INTO stories (name, owner)
            VALUES ($1, $2)
            RETURNING id, name, owner
        "#;

        let story = sqlx::query_as(sql)
            .bind(&name)
            .bind(&owner)
            .fetch_one(self.db_ref())
            .await?;

        Ok(story)
    }

    /// Select stories for an owner
    pub async fn select_stories(&self, owner: String) -> Result<Vec<Story>> {
        log::debug!("Repo::select_stories: {}", &owner);

        let sql = r#"
            SELECT id, name, owner
            FROM stories
            WHERE owner = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
        "#;

        let mut result_set = sqlx::query(sql).bind(&owner).fetch(self.db_ref());
        let mut result = Vec::new();

        while let Some(row) = result_set.try_next().await? {
            let story = Story::from_row(&row)?;
            result.push(story);
        }

        Ok(result)
    }

    /// Get a task by id
    pub async fn get_task(&self, task_id: Uuid) -> Result<Task> {
        log::debug!("Repo::get_task: {}", &task_id);

        let sql =
            "SELECT id, story_id, name, status FROM tasks WHERE id = $1 AND deleted_at IS NULL";

        let task = sqlx::query_as(sql)
            .bind(&task_id)
            .fetch_one(self.db_ref())
            .await?;

        Ok(task)
    }

    /// Insert a new story task
    pub async fn insert_task(&self, story_id: Uuid, name: String) -> Result<Task> {
        log::debug!("Repo::insert_task: {}, {}", &story_id, &name);

        let sql = r#"
            INSERT INTO tasks (story_id, name)
            VALUES ($1, $2)
            RETURNING id, story_id, name, status
        "#;

        let task = sqlx::query_as(sql)
            .bind(&story_id)
            .bind(&name)
            .fetch_one(self.db_ref())
            .await?;

        Ok(task)
    }

    /// Select tasks for a story
    pub async fn select_tasks(&self, story_id: Uuid) -> Result<Vec<Task>> {
        log::debug!("Repo::select_tasks: story: {}", &story_id);

        let sql = r#"
            SELECT id, story_id, name, status
            FROM tasks
            WHERE story_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
        "#;

        let mut result_set = sqlx::query(sql).bind(&story_id).fetch(self.db_ref());
        let mut result = Vec::new();

        while let Some(row) = result_set.try_next().await? {
            let task = Task::from_row(&row)?;
            result.push(task);
        }

        Ok(result)
    }

    /// Update task status
    pub async fn update_task_status(&self, task_id: Uuid, status: Status) -> Result<u64> {
        log::debug!("Repo::update_task_status: {}, {}", &task_id, &status);

        let status = status.to_string();

        let result = sqlx::query("UPDATE tasks SET status = $1, updated_at = now() WHERE id = $2")
            .bind(&status)
            .bind(&task_id)
            .execute(self.db_ref())
            .await?;

        Ok(result.rows_affected())
    }

    /// Delete a story by setting the deleted_at timestamp.
    pub async fn delete_story(&self, story_id: Uuid) -> Result<u64> {
        log::debug!("Repo::delete_story: {}", &story_id);

        let mut tx = self.db.begin().await?;

        let sql1 = r#"
            UPDATE tasks SET deleted_at = now() WHERE story_id = $1
            AND deleted_at IS NULL
        "#;
        let r1 = sqlx::query(sql1).bind(&story_id).execute(&mut *tx).await?;

        let sql2 = r#"
            UPDATE stories SET deleted_at = now() WHERE id = $1
            AND deleted_at IS NULL
        "#;
        let r2 = sqlx::query(sql2).bind(&story_id).execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(r1.rows_affected() + r2.rows_affected())
    }

    /// Delete a task by setting the deleted_at timestamp.
    pub async fn delete_task(&self, task_id: Uuid) -> Result<u64> {
        log::debug!("Repo::delete_task: {}", &task_id);

        let sql = r#"
            UPDATE tasks SET deleted_at = now()
            WHERE id = $1
            AND deleted_at IS NULL
        "#;

        let result = sqlx::query(sql)
            .bind(&task_id)
            .execute(self.db_ref())
            .await?;

        Ok(result.rows_affected())
    }
}
