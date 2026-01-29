use async_trait::async_trait;
use nimble_web::data::paging::Page;
use nimble_web::data::provider::{DataProvider, DataResult};
use nimble_web::data::query::{Query, Value};
use nimble_web::data::repository::Repository;
use nimble_web::entity::entity::Entity;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestUser {
    id: String,
    email: String,
}

impl Entity for TestUser {
    type Id = String;
    fn id(&self) -> &Self::Id {
        &self.id
    }
    fn name() -> &'static str {
        "User"
    }
}

struct MockUserProvider {
    user: TestUser,
}

#[async_trait]
impl DataProvider<TestUser> for MockUserProvider {
    async fn create(&self, e: TestUser) -> DataResult<TestUser> {
        Ok(e)
    }
    async fn get(&self, _id: &String) -> DataResult<Option<TestUser>> {
        Ok(None)
    }
    async fn update(&self, e: TestUser) -> DataResult<TestUser> {
        Ok(e)
    }
    async fn delete(&self, _id: &String) -> DataResult<bool> {
        Ok(true)
    }
    async fn query(&self, _q: Query<TestUser>) -> DataResult<Page<TestUser>> {
        Ok(Page::new(vec![self.user.clone()], 1, 1, 10))
    }
    async fn get_by(&self, column: &str, value: Value) -> DataResult<Option<TestUser>> {
        if column == "email" {
            if let Value::String(v) = value {
                if v == self.user.email {
                    return Ok(Some(self.user.clone()));
                }
            }
        }
        Ok(None)
    }
}

#[tokio::test]
async fn test_repository_get_by_delegation() {
    let user = TestUser {
        id: "1".to_string(),
        email: "test@example.com".to_string(),
    };
    let provider = MockUserProvider { user: user.clone() };
    let repo = Repository::new(Box::new(provider));

    // Test successful hit
    let found = repo
        .get_by("email", Value::String("test@example.com".to_string()))
        .await
        .unwrap();
    assert_eq!(found, Some(user));

    // Test miss
    let not_found = repo
        .get_by("email", Value::String("other@example.com".to_string()))
        .await
        .unwrap();
    assert!(not_found.is_none());

    // Test other column
    let other_col = repo
        .get_by("name", Value::String("test".to_string()))
        .await
        .unwrap();
    assert!(other_col.is_none());
}
