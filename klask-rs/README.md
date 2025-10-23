# Klask-RS: Modern Code Search Engine

A high-performance code search engine built with Rust, featuring real-time indexing, advanced search capabilities, and modern web technologies.

## 🚀 Quick Start

### Prerequisites
- Rust (1.70+)
- Docker & Docker Compose
- Git
- sqlx (cargo install sqlx-cli --no-default-features --features rustls,postgres)

### Development Setup

1. **Clone and setup**:
   ```bash
   git checkout rust-modernization
   cd klask-rs
   ```

2. **Start the database**:
   ```bash
   make setup
   ```

3. **Run the application**:
   ```bash
   make dev
   ```

4. **Access the services**:
   - API Server: http://localhost:3000
   - Health Check: http://localhost:3000/health
   - pgAdmin: http://localhost:8080 (admin@klask.dev / admin)

## 🏗️ Architecture

### Technology Stack
- **Backend**: Rust + Axum + Tokio
- **Database**: PostgreSQL + SQLx
- **Search**: Tantivy (Rust-native search engine)
- **Repository Integration**: Git2, GitLab API

### Project Structure
```
klask-rs/
├── src/
│   ├── api/           # REST API endpoints
│   ├── database/      # Database connection and migrations
│   ├── models/        # Data models
│   ├── repositories/  # Database operations
│   ├── services/      # Business logic
│   └── main.rs        # Application entry point
├── migrations/        # Database schema migrations
├── tests/            # Integration tests
└── docker-compose.yml # Development environment
```

## 📚 API Documentation

### Core Endpoints

**Files**
- `GET /api/files` - List files with pagination
- `GET /api/files/{id}` - Get file details

**Search**
- `GET /api/search?query=...` - Search code with filters

**Repositories**
- `GET /api/repositories` - List configured repositories
- `POST /api/repositories` - Add new repository
- `POST /api/repositories/{id}/crawl` - Trigger crawling

## 🛠️ Development Commands

```bash
# Database operations
make dev-db          # Start PostgreSQL only
make dev-full        # Start all services (DB + tools)
make migrate         # Run database migrations

# Development
make dev             # Start development server
make test            # Run tests
make build           # Build release version

# Docker operations
make docker-up       # Start all Docker services
make docker-down     # Stop Docker services
make clean           # Clean everything
```

## 🗄️ Database Schema

### Tables
- **users**: Authentication and user management
- **repositories**: Source repository configuration
- **files**: Indexed file metadata and content

### Environment Variables
```bash
DATABASE_URL=postgres://klask:klask@localhost:5432/klask_rs
HOST=127.0.0.1
PORT=3000
JWT_SECRET=your-secret-key
SEARCH_INDEX_DIR=./index
```

## 🔍 Search Features (In Development)

- **Full-text search**: Powered by Tantivy search engine
- **Syntax highlighting**: Code-aware result highlighting
- **Advanced filtering**: By language, project, version
- **Real-time indexing**: Automatic updates on code changes

## 🚢 Deployment

The application is designed to be deployed with Docker:

```bash
# Build production image
docker build -t klask-rs .

# Run with external database
docker run -d \
  -p 3000:3000 \
  -e DATABASE_URL=postgres://... \
  klask-rs
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License.
