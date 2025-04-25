# A Rusty POS API🦀

A simple POS API built using Rust and Axum. It manages products, categories, and transactions while using JWT for authentication. It used PostgreSQL for database storage, and S3 for file storage.

## 🚀 Getting Started

### Prerequisites
Ensure you have the following setup and installed.
- Rust & Cargo
- PostgreSQL
- An S3-compatible storage service (e.g., AWS S3, MinIO)

### Environment Variables
Ensure that the following environment variables are set up in a ``.env`` file.
```
SERVER_ADDRESS=127.0.0.1:3000
DATABASE_URL=postgres://user:password@localhost/database
JWT_SECRET=your_jwt_secret
S3_ENDPOINT=https://s3.yourprovider.com
S3_ACCESS_KEY=your_access_key
S3_SECRET_KEY=your_secret_key
S3_BUCKET=your_bucket_name
S3_REGION=your_region
```

### Running the Server
Using Cargo
```
git clone https://github.com/edions/simple-pos-api.git
cd simple-pos-api
cargo run
```

Using Docker
```
git clone https://github.com/edions/simple-pos-api.git
cd simple-pos-api
cargo sqlx prepare
docker build .
docker run -p 3000:3000 pos-api
```
The server will be accessible at `http://localhost:3000` (or another configured port).

## 📚 API Endpoints
> **_NOTE:_** Routes marked with a lock 🔒 are protected by JWT.

### Root Endpoint
- `GET /` - Returns "Server is running" to indicate the API is active.

### Authentication Routes
- `POST /api/login` - Authenticate a user and return a JWT token.
- `POST /api/signup` - Register a new user.

### Product Routes
- `GET /api/product` - Retrieve all products. 🔒
- `POST /api/product` - Create a new product. 🔒
- `GET /api/product/:product_id` - Retrieve a specific product by ID. 🔒
- `PATCH /api/product/:product_id` - Update product details. 🔒
- `DELETE /api/product/:product_id` - Delete a product. 🔒

### Category Routes
- `GET /api/category` - Retrieve all categories. 🔒
- `POST /api/category` - Create a new category. 🔒
- `PATCH /api/category/:category_id` - Update category details. 🔒
- `DELETE /api/category/:category_id` - Delete a category. 🔒

### Transaction Routes
- `GET /api/transaction` - Retrieve all transactions. 🔒
- `POST /api/transaction` - Record a new transaction. 🔒

## License
This project is licensed under the MIT License.
