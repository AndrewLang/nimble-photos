use uuid::Uuid;

pub struct IdGenerationService;

impl IdGenerationService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }

    pub fn generate_string(&self) -> String {
        self.generate().to_string()
    }
}
