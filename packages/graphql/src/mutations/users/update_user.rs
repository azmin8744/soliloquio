use super::validation_errors_to_message;
use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::UpdateUserInput;
use crate::types::user::User;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Object, Result, Union};
use repositories::UserRepository;
use sea_orm::*;
use services::email::EmailService;
use services::validation::input_validator::InputValidator;
use services::verification_token::{create_token, TokenKind};

#[derive(Union)]
pub enum UpdateUserResult {
    User(User),
    ValidationError(ValidationErrorType),
    AuthError(AuthError),
    DbError(DbError),
}

#[derive(Default)]
pub struct UpdateUserMutation;

impl RequiresAuth for UpdateUserMutation {}

#[Object]
impl UpdateUserMutation {
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        input: UpdateUserInput,
    ) -> Result<UpdateUserResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let current_user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => return Ok(UpdateUserResult::AuthError(AuthError { message: e.to_string() })),
        };

        if let Err(validation_errors) = input.validate() {
            return Ok(UpdateUserResult::ValidationError(ValidationErrorType {
                message: validation_errors_to_message(validation_errors),
            }));
        }

        if let Ok(Some(existing)) = UserRepository::find_by_email(db, &input.email).await {
            if existing.id != current_user.id {
                return Ok(UpdateUserResult::AuthError(AuthError {
                    message: "Email already in use".to_string(),
                }));
            }
        }

        let user_id = current_user.id;
        let email_changed = current_user.email != input.email;
        let updated =
            match UserRepository::update_email(db, current_user, input.email.clone(), email_changed)
                .await
            {
                Ok(u) => u,
                Err(e) => return Ok(UpdateUserResult::DbError(DbError { message: e.to_string() })),
            };

        if email_changed {
            if let Ok(email_service) = ctx.data::<EmailService>() {
                match create_token(db, user_id, TokenKind::EmailVerification, 86400).await {
                    Ok(raw_token) => {
                        if let Err(e) = email_service
                            .send_email_verification(&input.email, &raw_token)
                            .await
                        {
                            tracing::warn!(user_id = %user_id, error = %e, "failed to send verification email after email change");
                        }
                    }
                    Err(e) => tracing::warn!(user_id = %user_id, error = %e.message, "failed to create verification token"),
                }
            }
        }

        let has_profile_update = input.display_name.is_some() || input.bio.is_some();
        let final_user = if has_profile_update {
            match UserRepository::update_profile(db, user_id, input.display_name, input.bio).await {
                Ok(u) => u,
                Err(e) => return Ok(UpdateUserResult::DbError(DbError { message: e.to_string() })),
            }
        } else {
            updated
        };

        tracing::info!(user_id = %user_id, "user updated");
        Ok(UpdateUserResult::User(User {
            id: final_user.id,
            email: final_user.email,
            email_verified_at: final_user.email_verified_at,
            display_name: final_user.display_name,
            bio: final_user.bio,
            created_at: final_user.created_at,
            updated_at: final_user.updated_at,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

    #[tokio::test]
    async fn test_update_user_updates_email() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_user");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let new_email = generate_unique_email("update_user_new");

        let query = format!(
            r#"mutation {{ updateUser(input: {{ email: "{}" }}) {{
                ... on User {{ id email }}
                ... on AuthError {{ message }}
                ... on ValidationErrorType {{ message }}
            }} }}"#,
            new_email
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        assert_eq!(data["updateUser"]["email"], new_email);

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_update_user_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation { updateUser(input: { email: "new@example.com" }) {
            ... on AuthError { message }
            ... on User { email }
        } }"#;

        let res = schema.execute(Request::new(query)).await;
        let data = res.data.into_json().unwrap();
        assert!(data["updateUser"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_update_user_invalid_email_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_user_invalid");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation { updateUser(input: { email: "not-an-email" }) {
            ... on ValidationErrorType { message }
            ... on User { email }
        } }"#;

        let res = schema
            .execute(Request::new(query).data(services::authentication::Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();
        assert!(data["updateUser"]["message"].as_str().is_some());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_update_user_taken_email_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email1 = generate_unique_email("update_taken_1");
        let email2 = generate_unique_email("update_taken_2");
        let user1 = create_test_user_with_password(&db, &email1, &valid_password()).await;
        let user2 = create_test_user_with_password(&db, &email2, &valid_password()).await;
        let token = create_access_token(&user1);

        let query = format!(
            r#"mutation {{ updateUser(input: {{ email: "{}" }}) {{
                ... on AuthError {{ message }}
                ... on User {{ email }}
            }} }}"#,
            email2
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();
        assert_eq!(data["updateUser"]["message"], "Email already in use");

        cleanup_test_user(&db, user1.id).await;
        cleanup_test_user(&db, user2.id).await;
    }

    #[tokio::test]
    async fn test_update_user_updates_display_name() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_display_name");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = format!(
            r#"mutation {{ updateUser(input: {{ email: "{}", displayName: "Test Name" }}) {{
                ... on User {{ displayName bio }}
                ... on AuthError {{ message }}
            }} }}"#,
            email
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        assert_eq!(data["updateUser"]["displayName"], "Test Name");

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_update_user_updates_bio() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_bio");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = format!(
            r#"mutation {{ updateUser(input: {{ email: "{}", bio: "My bio" }}) {{
                ... on User {{ displayName bio }}
                ... on AuthError {{ message }}
            }} }}"#,
            email
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        assert_eq!(data["updateUser"]["bio"], "My bio");

        cleanup_test_user(&db, user.id).await;
    }
}
