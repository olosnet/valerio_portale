use crate::statics::models::StaticsResponse;
use crate::statics::repos::StaticsRepository;

pub struct StaticsService {
    repository: StaticsRepository,
}

impl StaticsService {
    pub fn new() -> Self {
        Self {
            repository: StaticsRepository::new(),
        }
    }

    pub fn get_enum_values(&self) -> StaticsResponse {
        StaticsResponse {
            tipo_strumentazione: self.repository.get_tipi_strumentazione().to_vec(),
            tipo_oggetto: self.repository.get_tipi_oggetto().to_vec(),
            costellazioni: self.repository.get_costellazioni().to_vec(),
            timezones: self.repository.get_timezones().to_vec(),
        }
    }
}
