use crate::v1::{
    entity::{Status, Story, Task},
    repo::Repo,
};
use crate::{validate::Validate, Error, Result};

pub struct Service {
    repo: Repo,
}

impl Service {
    pub fn new(repo: Repo) -> Self {
        Self { repo }
    }
}

impl Service {
    /// Validate input name and owner then create a new story
    pub async fn create_story(&self, name: &str, owner: &str) -> Result<Story> {
        log::debug!("Service::create_story: {}, {}", name, owner);

        self.repo
            .insert_story(
                Validate::non_empty(name, "name")?,
                Validate::non_empty(owner, "owner")?,
            )
            .await
    }

    /// Get owner stories
    pub async fn get_stories(&self, owner: &str) -> Result<Vec<Story>> {
        log::debug!("Service::get_stories: {}", owner);

        self.repo
            .select_stories(Validate::non_empty(owner, "owner")?)
            .await
    }

    /// Create a new task
    pub async fn create_task(&self, story_id: &str, name: &str) -> Result<Task> {
        log::debug!("Service::create_task: {}, {}", story_id, name);

        self.repo
            .insert_task(
                Validate::validate_uuid(story_id)?,
                Validate::non_empty(name, "name")?,
            )
            .await
    }

    /// Get a task by id
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        log::debug!("Service::get_task: {}", task_id);

        self.repo.get_task(Validate::validate_uuid(task_id)?).await
    }

    /// Get all stories for an owner
    pub async fn get_tasks(&self, story_id: &str) -> Result<Vec<Task>> {
        log::debug!("Service::get_tasks: {}", story_id);

        self.repo
            .select_tasks(Validate::validate_uuid(story_id)?)
            .await
    }

    /// Mark a task as complete
    pub async fn complete_task(&self, task_id: &str) -> Result<()> {
        log::debug!("Service::complete_task: {}", task_id);

        let rows_affected = self
            .repo
            .update_task_status(Validate::validate_uuid(task_id)?, Status::Complete)
            .await?;

        if rows_affected == 0 {
            return Err(Error::NotFoundError {
                message: format!("unable to complete task: {}", task_id),
            });
        }

        Ok(())
    }

    /// Delete a story
    pub async fn delete_story(&self, story_id: &str) -> Result<()> {
        log::debug!("Service::delete_story: {}", story_id);

        let rows_affected = self
            .repo
            .delete_story(Validate::validate_uuid(story_id)?)
            .await?;

        if rows_affected == 0 {
            return Err(Error::NotFoundError {
                message: format!("unable to delete story: {}", story_id),
            });
        }

        Ok(())
    }

    /// Delete a task
    pub async fn delete_task(&self, task_id: &str) -> Result<()> {
        log::debug!("Service::delete_task: {}", task_id);

        let rows_affected = self
            .repo
            .delete_task(Validate::validate_uuid(task_id)?)
            .await?;

        if rows_affected == 0 {
            return Err(Error::NotFoundError {
                message: format!("unable to delete task: {}", task_id),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::repo::Repo;

    use sqlx::migrate::Migrator;
    use sqlx::postgres::{PgPool, PgPoolOptions};
    use std::path::Path;
    use std::sync::Arc;

    use testcontainers::{clients::Cli, Container, RunnableImage};
    use testcontainers_modules::postgres::Postgres;

    async fn setup_pg_pool(container: &Container<'_, Postgres>) -> Arc<PgPool> {
        let connection_string = &format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            container.get_host_port_ipv4(5432),
        );

        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&connection_string)
            .await
            .unwrap();

        let m = Migrator::new(Path::new("./migrations")).await.unwrap();
        m.run(&pool).await.unwrap();

        Arc::new(pool)
    }

    #[ignore]
    #[tokio::test]
    async fn service_integration_test() {
        env_logger::init();

        // Set up postgres test container backed repo
        let docker = Cli::default();
        let image = RunnableImage::from(Postgres::default()).with_tag("15-alpine");
        let container = docker.run(image);
        let pool = setup_pg_pool(&container).await;

        // Create service - test public database logic
        let service = Service::new(Repo::new(pool));

        // Create story
        let name = "Books To Read";
        let owner = "github.com/carp-cobain";
        let story = service.create_story(name, owner).await.unwrap();
        assert_eq!(name, story.name);
        let story_id = &story.story_id.to_string();

        // Query stories for owner
        let stories = service.get_stories(owner).await.unwrap();
        assert_eq!(stories.len(), 1);

        // Create task, ensuring initial status is "incomplete"
        let task_name = "Blood Meridian";
        let task = service.create_task(story_id, task_name).await.unwrap();
        assert_eq!(task.status, Status::Incomplete);
        let task_id = &task.task_id.to_string();

        // Query tasks for story
        let tasks = service.get_tasks(story_id).await.unwrap();
        assert_eq!(tasks.len(), 1);

        // Complete task, query, and assert completed
        service.complete_task(task_id).await.unwrap();
        let task = service.get_task(task_id).await.unwrap();
        assert_eq!(task.status, Status::Complete);

        // Delete the story
        let result = service.delete_task(task_id).await.unwrap();
        assert_eq!(result, ());

        // Delete the story (and repeat deleting the task)
        let result = service.delete_story(story_id).await.unwrap();
        assert_eq!(result, ());
    }
}
