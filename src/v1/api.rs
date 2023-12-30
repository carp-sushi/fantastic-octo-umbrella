use crate::v1::proto::todos_service_server::TodosService;
use crate::v1::proto::*;
use crate::v1::service::Service;
use crate::Error;
use tonic::{Request, Response, Status};

/// Todos presentation layer (gRPC).
pub struct Todos {
    service: Service,
}

impl Todos {
    /// Todos constructor
    pub fn new(service: Service) -> Self {
        Self { service }
    }
}

/// Map the error type to grpc status.
impl From<Error> for Status {
    fn from(err: Error) -> Self {
        match err {
            Error::InvalidArgument { message } => Status::invalid_argument(message),
            Error::InternalError { message } => Status::internal(message),
            Error::NotFoundError { message } => Status::not_found(message),
        }
    }
}

/// Map entity story to presentation type
impl From<crate::v1::entity::Story> for Story {
    fn from(entity: crate::v1::entity::Story) -> Self {
        Self {
            story_id: entity.story_id.to_string(),
            name: entity.name,
            owner: entity.owner,
        }
    }
}

/// Map entity task into presentation type
impl From<crate::v1::entity::Task> for Task {
    fn from(entity: crate::v1::entity::Task) -> Self {
        Self {
            task_id: entity.task_id.to_string(),
            story_id: entity.story_id.to_string(),
            name: entity.name,
            complete: entity.status == crate::v1::entity::Status::Complete,
        }
    }
}

#[tonic::async_trait]
impl TodosService for Todos {
    /// Create a new story
    async fn create_story(
        &self,
        request: Request<CreateStoryRequest>,
    ) -> Result<Response<CreateStoryResponse>, Status> {
        log::info!("Create story request from {:?}", request.remote_addr());

        let request = request.get_ref();
        let entity = self
            .service
            .create_story(&request.name, &request.owner)
            .await?;

        Ok(Response::new(CreateStoryResponse {
            story: Some(entity.into()),
        }))
    }

    /// Get owner stories
    async fn get_stories(
        &self,
        request: Request<GetStoriesRequest>,
    ) -> Result<Response<GetStoriesResponse>, Status> {
        log::info!("Get stories request from {:?}", request.remote_addr());

        let stories = self
            .service
            .get_stories(&request.get_ref().owner)
            .await?
            .into_iter()
            .map(|s| s.into())
            .collect();

        Ok(Response::new(GetStoriesResponse { stories }))
    }

    /// Create a new task
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        log::info!("Create task request from {:?}", request.remote_addr());

        let request = request.get_ref();
        let entity = self
            .service
            .create_task(&request.story_id, &request.name)
            .await?;

        Ok(Response::new(CreateTaskResponse {
            task: Some(entity.into()),
        }))
    }

    /// Get all tasks for a story
    async fn get_tasks(
        &self,
        request: Request<GetTasksRequest>,
    ) -> Result<Response<GetTasksResponse>, Status> {
        log::info!("Get tasks request from {:?}", request.remote_addr());

        let tasks = self
            .service
            .get_tasks(&request.get_ref().story_id)
            .await?
            .into_iter()
            .map(|t| t.into())
            .collect();

        Ok(Response::new(GetTasksResponse { tasks }))
    }

    /// Complete a task
    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> Result<Response<CompleteTaskResponse>, Status> {
        log::info!("Complete task request from {:?}", request.remote_addr());

        self.service
            .complete_task(&request.get_ref().task_id)
            .await?;

        Ok(Response::new(CompleteTaskResponse {}))
    }

    /// Delete a story
    async fn delete_story(
        &self,
        request: Request<DeleteStoryRequest>,
    ) -> Result<Response<DeleteStoryResponse>, Status> {
        log::info!("Delete story request from {:?}", request.remote_addr());

        self.service
            .delete_story(&request.get_ref().story_id)
            .await?;

        Ok(Response::new(DeleteStoryResponse {}))
    }

    /// Delete a tast
    async fn delete_task(
        &self,
        request: Request<DeleteTaskRequest>,
    ) -> Result<Response<DeleteTaskResponse>, Status> {
        log::info!("Delete task request from {:?}", request.remote_addr());

        self.service.delete_task(&request.get_ref().task_id).await?;

        Ok(Response::new(DeleteTaskResponse {}))
    }
}
