use super::{UserMutation, UserMutationResult, validation_errors_to_message};
use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::UpdateUserInput;
use crate::types::user::User;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result};
use models::{prelude::*, *};
use sea_orm::*;
use services::email::EmailService;
use services::validation::input_validator::InputValidator;
use services::verification_token::{create_token, TokenKind};

pub(super) async fn update_user(
    mutation: &UserMutation,
    ctx: &Context<'_>,
    input: UpdateUserInput,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let current_user = match mutation.require_authenticate_as_user(ctx).await {
        Ok(user) => user,
        Err(e) => return Ok(UserMutationResult::AuthError(AuthError { message: e.to_string() })),
    };

    if let Err(validation_errors) = input.validate() {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: validation_errors_to_message(validation_errors),
        }));
    }

    if let Ok(Some(existing)) = Users::find()
        .filter(users::Column::Email.eq(&input.email))
        .one(db)
        .await
    {
        if existing.id != current_user.id {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Email already in use".to_string(),
            }));
        }
    }

    let user_id = current_user.id;
    let email_changed = current_user.email != input.email;
    let mut user_active = current_user.into_active_model();
    user_active.email = ActiveValue::set(input.email.clone());
    user_active.updated_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
    if email_changed {
        user_active.email_verified_at = ActiveValue::set(None);
    }

    let updated = match Users::update(user_active).exec(db).await {
        Ok(u) => u,
        Err(e) => return Ok(UserMutationResult::DbError(DbError { message: e.to_string() })),
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

    tracing::info!(user_id = %user_id, "user email updated");
    Ok(UserMutationResult::User(User {
        id: updated.id,
        email: updated.email,
        email_verified_at: updated.email_verified_at,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    }))
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
}
