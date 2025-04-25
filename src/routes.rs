use std::sync::Arc;
use axum::{http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, Method, StatusCode}, middleware, response::IntoResponse, routing::{get, patch, post}, Router};
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

use crate::{
    handlers::{
        auth::{login, signup},
        category::{create_category, delete_category, get_all_categories, update_category},
        product::{create_product, delete_product, get_all_products, get_product, update_product},
        transaction::{create_transaction, get_all_transactions}
    },
    middlewares::auth_guard::auth,
    AppState
};

pub fn app_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(root))
            .nest("/api", auth_route(app_state.clone()))
            .nest("/api/product", product_route(app_state.clone()))
            .nest("/api/category", category_route(app_state.clone()))
            .nest("/api/transaction", transaction_route(app_state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::DELETE,
                ])
            .allow_headers([
                AUTHORIZATION,
                CONTENT_TYPE,
                ACCEPT,
                ]))
        .fallback(handler_404)
}

async fn root() -> &'static str {
    "Server is running"
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "The requested resource was not found"
    )
}
async fn handle_405() -> impl IntoResponse {
    "Method not allowed fallback"
}

pub fn auth_route(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .with_state(app_state)
        .method_not_allowed_fallback(handle_405)
}

pub fn product_route(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_products).post(create_product))
        .route("/{product_id}", get( get_product)
            .patch(update_product)
            .delete(delete_product))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
        .method_not_allowed_fallback(handle_405)
}

pub fn category_route(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_categories).post(create_category))
        .route("/{category_id}", patch(update_category).delete(delete_category))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
        .method_not_allowed_fallback(handle_405)
}

pub fn transaction_route(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_all_transactions).post(create_transaction))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .with_state(app_state)
        .method_not_allowed_fallback(handle_405)
}