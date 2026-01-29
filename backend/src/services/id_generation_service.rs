use uuid::Uuid;

pub struct IdGenerationService;

impl IdGenerationService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self) -> String {
        Uuid::new_v4().to_string()
    }
}
