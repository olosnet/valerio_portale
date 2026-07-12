use crate::permissions::repos::PermissionsRepository;

pub struct PermissionsService<'a> {
    repository: PermissionsRepository<'a>,
}

impl<'a> PermissionsService<'a> {
    pub fn new(repository: PermissionsRepository<'a>) -> Self {
        PermissionsService { repository }
    }

    pub async fn list_permissions(
        &self,
    ) -> Result<Vec<String>, cornetti::core::models::CornettiError> {
        self.repository.list().await
    }
}
