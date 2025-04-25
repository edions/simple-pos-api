use axum::{extract::{Multipart, Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

use crate::{
    models::{
        filter_model::FilterOptionsModel,
        products_model::{GetProductModel, PostProductModel}},
    services::image_service::upload_image,
    AppState
};

pub async fn get_all_products(
    State(app_state): State<Arc<AppState>>,
    Query(filter_options): Query<FilterOptionsModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    // let Query(opts) = filter_options.unwrap_or_default();

    let limit = filter_options.limit.unwrap_or(10);
    let offset = (filter_options.offset.unwrap_or(1) - 1) * limit;

    let total_products: Option<i64> = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*)
            FROM products
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

    let mut products = sqlx::query_as!(
        GetProductModel,
        r#"
            SELECT
                product_id, product_name, price, stock, sku, category_name, product_image, products.created_at, products.updated_at
            FROM products
            LEFT JOIN categories
            ON products.category_id = categories.category_id
            ORDER BY product_id
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
                    "success" : false,
                    "message" : e.to_string(),
                })),
            )
        })?;
    
    for product in &mut products {
        if let Some(image) = &product.product_image {
            let presign_url = app_state.s3.presign_get(image, 86400, None).await.map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "success": false,
                        "message": "Failed to generate presigned URL",
                    })),
                )
            })?;

            product.product_image = Some(presign_url);
        }
    }
    
    let json_response = json!({
        "success" : true,
        "data" : products,
        "total": total_products,
        "limit": limit,
        "offset": offset,
    });

    Ok((
        StatusCode::OK,
        Json(json_response),
    ))
}

pub async fn get_product(
    State(app_state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    let mut product = sqlx::query_as!(
        GetProductModel,
        r#"
            SELECT
                product_id, product_name, price, stock, sku, category_name, product_image, products.created_at, products.updated_at
            FROM products
            LEFT JOIN categories
            ON products.category_id = categories.category_id
            WHERE product_id = $1
        "#,
        product_id
    )
    .fetch_one(&app_state.db)
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

    if let Some(image) = &product.product_image {
        let presign_url = app_state.s3.presign_get(image, 86400, None).await.map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "success": false,
                "message": "Failed to generate presigned URL",
            })))
        })?;
        
        product.product_image = Some(presign_url);
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": false,
            "data": product,
        })),
    ))
}

pub async fn create_product(
    State(app_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    let mut product = PostProductModel {
        product_id: None,
        product_name: None,
        price: None,
        stock: None,
        sku: None,
        category_id: None,
        product_image: None,
        created_at: None,
        updated_at: None,
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("product_name") => {
                if let Ok(text) = field.text().await {
                    product.product_name = Some(text);
                }
            }
            Some("price") => {
                if let Ok(price_str) = field.text().await {
                    product.price = Some(price_str.parse::<Decimal>().unwrap_or(Decimal::new(0, 0)));
                }
            }
            Some("stock") => {
                if let Ok(stock_str) = field.text().await {
                    product.stock = Some(stock_str.parse::<i32>().unwrap_or(0));
                }
            }
            Some("sku") => {
                if let Ok(text) = field.text().await {
                    product.sku = Some(text);
                }
            }
            Some("category_id") => {
                if let Ok(id_str) = field.text().await {
                    product.category_id = Some(id_str);
                }
            }
            Some("product_image") => {
                product.product_image = Some(upload_image(field, &app_state).await?);
            }
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Unexpected field found in form data"
                    })),
                ));
            }
        }
    }

    let product_id = data_encoding::BASE64URL_NOPAD.encode( Uuid::new_v4().as_bytes());

    let product = sqlx::query_as!(
        PostProductModel,
        r#"
            INSERT INTO products (product_id, product_name, price, stock, sku, category_id, product_image, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#,
        product_id,
        product.product_name,
        product.price,
        product.stock,
        product.sku,
        product.category_id,
        product.product_image,
        Utc::now(),
        Utc::now(),
    )
    .fetch_one(&app_state.db)
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
        StatusCode::CREATED,
        Json(json!({
            "success": true,
            "data": product,
        })),
    ))
}

pub async fn update_product(
    State(app_state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
    mut multipart: Multipart,
) ->  Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    let mut update_product = PostProductModel {
        product_id: None,
        product_name: None,
        price: None,
        stock: None,
        sku: None,
        category_id: None,
        product_image: None,
        created_at: None,
        updated_at: None,
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("product_name") => {
                if let Ok(text) = field.text().await {
                    update_product.product_name = Some(text);
                }
            }
            Some("price") => {
                if let Ok(price_str) = field.text().await {
                    if let Ok(price) = price_str.parse::<Decimal>() {
                        update_product.price = Some(price);
                    }
                }
            }
            Some("stock") => {
                if let Ok(stock_str) = field.text().await {
                    if let Ok(stock) = stock_str.parse::<i32>() {
                        update_product.stock = Some(stock);
                    }
                }
            }
            Some("sku") => {
                if let Ok(text) = field.text().await {
                    update_product.sku = Some(text);
                }
            }
            Some("category_id") => {
                if let Ok(id_str) = field.text().await {
                    update_product.category_id = Some(id_str);
                }
            }
            Some("product_image") => {
                if let Ok(image) = upload_image(field, &app_state).await {
                    update_product.product_image = Some(image)
                }
            }
            _ => {
                continue;
            }
        }
    }

    sqlx::query!(
        r#"
            UPDATE products
            SET
                product_name = COALESCE($1, product_name),
                price = COALESCE($2, price),
                stock = COALESCE($3, stock),
                sku = COALESCE($4, sku),
                category_id = COALESCE($5, category_id),
                product_image = COALESCE($6, product_image),
                updated_at = COALESCE($7, updated_at)
            WHERE product_id = $8
        "#,
        update_product.product_name,
        update_product.price,
        update_product.stock,
        update_product.sku,
        update_product.category_id,
        update_product.product_image,
        Utc::now(),
        product_id,
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

pub async fn delete_product(
    State(app_state): State<Arc<AppState>>,
    Path(product_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    sqlx::query!(
        r#"
            DELETE FROM products
            WHERE product_id = $1
        "#,
        product_id,
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
            "success":true,
        })),
    ))
}
