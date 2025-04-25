use std::sync::Arc;
use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;

use crate::{models::{categories_model::
    CategoryModel, filter_model::FilterOptionsModel},
    AppState
};

pub async fn get_all_categories(
    State(app_state): State<Arc<AppState>>,
    Query(filter_options): Query<FilterOptionsModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)>{

    // let Query(opts) = filter_options.unwrap_or_default();

    let limit = filter_options.limit.unwrap_or(10);
    let offset = (filter_options.offset.unwrap_or(1) - 1) * limit;

    let total_categories: Option<i64> = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*)
            FROM categories
        "#
    )
    .fetch_one(&app_state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success" : false,
                "message" : e.to_string(),
            })),
        )
    })?;
    
    let categories = sqlx::query_as!(
        CategoryModel,
        r#"
            SELECT category_id, category_name, created_at, updated_at
            FROM categories
            OFFSET $1
            LIMIT $2
        "#,
        offset,
        limit,
    )
        .fetch_all(&app_state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": e.to_string(),
                })),
            )
        })?;
    
    let json_response = json!({
        "success": true,
        "data": categories,
        "total": total_categories,
        "offset": offset,
        "limit": limit,
    });

    Ok((
        StatusCode::OK,
        Json(json_response),
    ))
}

pub async fn create_category(
    State(app_state): State<Arc<AppState>>,
    Json(category): Json<CategoryModel> 
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    let category_id = data_encoding::BASE64URL_NOPAD.encode( Uuid::new_v4().as_bytes());

    let category = sqlx::query_as!(
        CategoryModel,
        r#"
            INSERT INTO categories (category_id, category_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
        "#,
        category_id,
        category.category_name,
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(&app_state.db)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "message": e.to_string(),
        })),
    ))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": category,
        })),
    ))
}

pub async fn update_category(
    State(app_state): State<Arc<AppState>>,
    Path(category_id): Path<String>,
    Json(update_category): Json<CategoryModel>
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    sqlx::query!(
        r#"
            UPDATE categories
            SET category_name = $1, created_at = $2
            WHERE category_id = $3
        "#,
        update_category.category_name,
        Utc::now(),
        category_id,
    )
        .execute(&app_state.db)
        .await
        .map_err(|e| {
            (
                 StatusCode::INTERNAL_SERVER_ERROR,
                 Json(json!({
                    "success": false,
                    "message": e.to_string(),
                })),
            )
         })?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
        }))
    ))
}

pub async fn delete_category(
    State(app_state): State<Arc<AppState>>,
    Path(category_id): Path<String>
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    sqlx::query!(
        r#"
            DELETE FROM categories
            WHERE category_id = $1
        "#,
        category_id
    )
        .execute(&app_state.db)
        .await
        .map_err(|e| {
           (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "message": e.to_string(),
                })),
           )
        })?;
    
    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
        })),
    ))
}