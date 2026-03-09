use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::StreamExt;
use sea_orm::DatabaseConnection;
use services::assets::StorageDriver;
use services::authentication::{authenticator::get_user, token::Token};
use std::sync::Arc;
use uuid::Uuid;

fn get_token(req: &HttpRequest) -> Option<Token> {
    if let Some(v) = req.headers().get("Authorization") {
        if let Ok(s) = v.to_str() {
            return Some(Token(s.to_string()));
        }
    }
    if let Some(v) = req.headers().get("cookie") {
        if let Ok(s) = v.to_str() {
            for pair in s.split(';') {
                let pair = pair.trim();
                if let Some(val) = pair.strip_prefix("access_token=") {
                    return Some(Token(val.to_string()));
                }
            }
        }
    }
    None
}

pub async fn upload(
    req: HttpRequest,
    mut multipart: Multipart,
    db: web::Data<DatabaseConnection>,
    driver: web::Data<Arc<StorageDriver>>,
) -> HttpResponse {
    let token = match get_token(&req) {
        Some(t) => t,
        None => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "missing token"})),
    };

    let user = match get_user(db.get_ref(), &token).await {
        Ok(u) => u,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": e.to_string()})),
    };

    // Read multipart field "file"
    while let Some(field) = multipart.next().await {
        let mut field = match field {
            Ok(f) => f,
            Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
        };

        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .and_then(|cd| cd.get_name())
            .unwrap_or("");

        if field_name != "file" {
            continue;
        }

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();

        let allowed_type = matches!(
            content_type.as_str(),
            "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/avif"
        );
        if !allowed_type {
            return HttpResponse::UnprocessableEntity()
                .json(serde_json::json!({"error": "only jpeg/png/gif/webp/avif accepted"}));
        }

        let original_filename = content_disposition
            .and_then(|cd| cd.get_filename())
            .unwrap_or("upload")
            .to_string();

        const MAX_BYTES: usize = 10 * 1024 * 1024; // 10 MiB
        let mut bytes = Vec::new();
        while let Some(chunk) = field.next().await {
            match chunk {
                Ok(data) => {
                    if bytes.len() + data.len() > MAX_BYTES {
                        return HttpResponse::PayloadTooLarge()
                            .json(serde_json::json!({"error": "image exceeds 10 MiB limit"}));
                    }
                    bytes.extend_from_slice(&data);
                }
                Err(e) => return HttpResponse::BadRequest().json(serde_json::json!({"error": e.to_string()})),
            }
        }

        let asset_id = Uuid::new_v4();
        let original_size = match services::assets::process_and_store(&bytes, asset_id, &driver).await {
            Ok(s) => s,
            Err(e) => return HttpResponse::UnprocessableEntity().json(serde_json::json!({"error": e})),
        };

        let asset = match repositories::AssetRepository::create(
            db.get_ref(),
            asset_id,
            user.id,
            original_filename,
            content_type.clone(),
            original_size as i64,
        )
        .await
        {
            Ok(a) => a,
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        };

        let base = format!("{asset_id}");
        let urls = serde_json::json!({
            "thumbnail": driver.url(&format!("{base}/thumbnail.webp")),
            "small": driver.url(&format!("{base}/small.webp")),
            "medium": driver.url(&format!("{base}/medium.webp")),
            "large": driver.url(&format!("{base}/large.webp")),
            "original": driver.url(&format!("{base}/original.webp")),
        });

        return HttpResponse::Ok().json(serde_json::json!({
            "id": asset.id,
            "originalFilename": asset.original_filename,
            "mimeType": asset.mime_type,
            "sizeBytes": asset.size_bytes,
            "urls": urls,
            "createdAt": asset.created_at,
        }));
    }

    HttpResponse::BadRequest().json(serde_json::json!({"error": "no file field found"}))
}

pub async fn serve_asset(
    path: web::Path<String>,
    driver: web::Data<Arc<StorageDriver>>,
) -> HttpResponse {
    let key = path.into_inner();
    match driver.get(&key).await {
        Ok(bytes) => HttpResponse::Ok()
            .content_type("image/webp")
            .insert_header(("X-Content-Type-Options", "nosniff"))
            .body(bytes),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}
