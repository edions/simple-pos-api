use std::sync::Arc;
use axum::{extract::{Query, State}, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::{
    models::{
        filter_model::FilterOptionsModel,
        transactions_model::{TransactionInputModel, TransactionModel}},
    AppState
};


pub async fn get_all_transactions(
    State(app_state): State<Arc<AppState>>,
    Query(filter_options): Query<FilterOptionsModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    // let Query(opts) = filter_options.unwrap_or_default();

    let limit = filter_options.limit.unwrap_or(20);
    let offset = (filter_options.offset.unwrap_or(1) - 1) * limit;

    let total_transactions: Option<i64> = sqlx::query_scalar!(
        r#"
            SELECT COUNT(*)
            FROM transactions
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
    
    let transactions = sqlx::query_as!(
        TransactionModel,
        r#"
            SELECT * FROM transactions
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
                    "succes": false,
                    "message": e.to_string(),
                })),
            )
        })?;
    
    let json_response = json!({
        "succes": true,
        "data": transactions,
        "total": total_transactions,
        "offset": offset,
        "limit": limit,
    });
    
    Ok((
        StatusCode::OK,
        Json(json_response),
    ))
}

pub async fn create_transaction(
    State(app_state): State<Arc<AppState>>,
    Json(transactions): Json<TransactionInputModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {

    // let total_price = transactions.total_price;
    let transaction_items = json!(transactions.transaction_items);

    let item_count = transaction_items
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    // let total_price = transactions.transaction_items.iter().fold(Decimal::ZERO, |acc, item| {
    //     acc + (Decimal::from_f64(item.price).unwrap() * Decimal::from(item.quantity))
    // });

    let total_price = transactions.transaction_items.iter().fold(Decimal::ZERO, |acc, item| {
        if let Some(price) = Decimal::from_f64(item.price) {
            acc + (price * Decimal::from(item.quantity))
        } else {
            acc
        }
    });

    let transaction_id = data_encoding::BASE64URL_NOPAD.encode( Uuid::new_v4().as_bytes());

    let result = sqlx::query_as!(
        TransactionModel,
        r#"
            INSERT INTO transactions (transaction_id, transaction_date, total_price, transaction_items, item_count)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
        "#,
        transaction_id,
        Utc::now(),
        total_price,
        transaction_items,
        item_count as i32,
    )
        .fetch_all(&app_state.db)
        .await
        .map_err(|e|{
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
            "data": result,
        })),
    ))
}