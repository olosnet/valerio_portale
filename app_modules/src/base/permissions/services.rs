use cornetti::core::models::CornettiResult;

use crate::base::permissions::repos::PermissionsRepository;

pub struct PermissionsService<'a> {
    repository: PermissionsRepository<'a>,
}

impl<'a> PermissionsService<'a> {
    pub fn new(repository: PermissionsRepository<'a>) -> Self {
        PermissionsService { repository }
    }

    pub async fn list_permissions(
        &self,
    ) -> CornettiResult<Vec<String>> {
        self.repository.list().await
    }
}
