use std::sync::Arc;

use cornetti::{core::models::CornettiResult, mongo::services::MongoDBService};
use validator::Validate;

use crate::base::enums::{
    models::{EnumCreate, EnumItem, EnumUpdate},
    repos::EnumsRepository,
};

pub struct EnumsService {
    repository: EnumsRepository,
}

impl EnumsService {
    pub fn new(mongo: Arc<MongoDBService>) -> Self {
        Self {
            repository: EnumsRepository::new(mongo),
        }
    }

    pub async fn list_enums(&self, category: Option<&str>) -> CornettiResult<Vec<EnumItem>> {
        self.repository.list(category).await
    }

    pub async fn get_enum(&self, enum_id: &str) -> CornettiResult<EnumItem> {
        self.repository.get(enum_id).await
    }

    pub async fn create_enum(&self, enum_create: EnumCreate) -> CornettiResult<EnumItem> {
        enum_create.validate()?;
        self.repository.create(enum_create).await
    }

    pub async fn update_enum(
        &self,
        enum_id: &str,
        enum_update: EnumUpdate,
    ) -> CornettiResult<EnumItem> {
        enum_update.validate()?;
        self.repository.update(enum_id, &enum_update).await
    }

    pub async fn delete_enum(&self, enum_id: &str) -> CornettiResult<()> {
        self.repository.delete(enum_id).await
    }
}