use nimble_web::pipeline::pipeline::PipelineError;

pub trait StringValidations {
    fn should_not_empty(self, field_name: &str) -> Result<Self, PipelineError>
    where
        Self: Sized;
}

impl<'a> StringValidations for &'a str {
    fn should_not_empty(self, field_name: &str) -> Result<Self, PipelineError> {
        if self.trim().is_empty() {
            Err(PipelineError::message(&format!(
                "{} should not be empty",
                field_name
            )))
        } else {
            Ok(self)
        }
    }
}

impl StringValidations for String {
    fn should_not_empty(self, field_name: &str) -> Result<Self, PipelineError> {
        if self.trim().is_empty() {
            Err(PipelineError::message(&format!(
                "{} should not be empty",
                field_name
            )))
        } else {
            Ok(self)
        }
    }
}
