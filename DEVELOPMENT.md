# Klask Development Setup

This guide will help you set up and test both the Rust backend and React frontend for the modernized Klask project.

## Prerequisites

### Required Software
- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Node.js** (18+) - [Install Node.js](https://nodejs.org/)
- **PostgreSQL** (14+) - For database
- **Git** - For version control

### Optional but Recommended
- **Docker & Docker Compose** - For easy PostgreSQL setup
- **VS Code** - With Rust and TypeScript extensions

## Quick Start

### 1. Database Setup (PostgreSQL)

#### Option A: Using Docker Compose (Recommended)
```bash
# Start PostgreSQL with Docker
docker compose -f docker-compose.dev.yml up -d
```

### 2. Rust Backend Setup

```bash
# Navigate to Rust project
cd klask-rs

# Copy environment file and configure
cp .env.example .env
# Edit .env with your database settings

# Install dependencies and run migrations
cargo build
sqlx migrate run

# Start the backend server
cargo run
```

The backend will start on **http://localhost:8080**

### 3. React Frontend Setup

```bash
# Navigate to React project (in a new terminal)
cd klask-react

# Install dependencies
npm install

# Start the development server
npm run dev
```

The frontend will start on **http://localhost:5173**

## Testing the Authentication

### Current Implementation Status
✅ **Frontend**: Login/Register forms with validation  
✅ **Backend**: JWT authentication with Argon2 password hashing  
⚠️ **Integration**: Ready for testing (API endpoints implemented)

### Testing Steps

1. **Start both servers** (backend on :8080, frontend on :5173)

2. **Test Registration**:
   - Go to http://localhost:5173/register
   - Fill the form with:
     - Username: `testuser`
     - First/Last Name: `Test User`
     - Email: `test@example.com`
     - Password: `TestPass123`
   - Submit and check browser console for API calls

3. **Test Login**:
   - Go to http://localhost:5173/login
   - Use credentials: `testuser` / `TestPass123`
   - Check if you're redirected to search page

4. **Verify in Database**:
   ```sql
   SELECT id, username, email, role, active, created_at 
   FROM users;
   ```

### API Endpoints Available

#### Authentication
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `GET /api/auth/profile` - Get user profile (requires JWT)

#### Health Check
- `GET /health` - Service health status

#### Search (Coming Next)
- `GET /api/search` - Search files and content
- `GET /api/search/filters` - Get available filters

#### Repositories (Admin)
- `GET /api/repositories` - List repositories
- `POST /api/repositories` - Add repository
- `POST /api/repositories/{id}/crawl` - Trigger crawl

## Environment Configuration

### Backend (.env)
```env
# Database
DATABASE_URL=postgresql://klask_user:klask_password@localhost/klask_dev

# Server
HOST=127.0.0.1
PORT=3000

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_EXPIRES_IN=24h

# Search Index
SEARCH_INDEX_PATH=./search_index

# Crawler
TEMP_DIR=./temp_crawl
```

### Frontend (.env)
```env
VITE_API_BASE_URL=http://localhost:3000
```

## Development Workflow

### Running Tests

#### Backend Tests
```bash
cd klask-rs
cargo test
```

#### Frontend Tests
```bash
cd klask-react
npm test
```

### Building for Production

#### Backend
```bash
cd klask-rs
cargo build --release
```

#### Frontend
```bash
cd klask-react
npm run build
```

## Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Check PostgreSQL is running
   - Verify DATABASE_URL in .env
   - Ensure database exists

2. **Frontend Can't Connect to Backend**
   - Verify backend is running on port 8080
   - Check CORS settings in backend
   - Confirm VITE_API_BASE_URL is correct

3. **JWT Token Issues**
   - Check JWT_SECRET is set
   - Verify token expiration settings
   - Clear browser localStorage if needed

4. **Search Index Errors**
   - Ensure SEARCH_INDEX_PATH directory exists
   - Check write permissions
   - Re-run crawler if index is corrupted

### Logs and Debugging

#### Backend Logs
```bash
cd klask-rs
RUST_LOG=debug cargo run
```

#### Frontend Console
- Open browser DevTools (F12)
- Check Console and Network tabs
- Look for API call responses

## Next Steps

### Current Development Plan
1. ✅ Authentication UI (completed)
2. 🔄 Search Interface (next)
3. 📋 Repository Management
4. 🎨 Syntax Highlighting
5. 🧪 E2E Testing

### Contributing
- Make changes in feature branches
- Run tests before committing
- Follow commit message format with Co-Authored-By tag
- Test both frontend and backend integration

## Architecture Overview

```
┌─────────────────┐    HTTP/JSON     ┌──────────────────┐
│   React App     │ ◄──────────────► │   Rust Backend   │
│   (port 5173)   │                  │   (port 8080)    │
│                 │                  │                  │
│ • React 18      │                  │ • Axum Framework │
│ • TypeScript    │                  │ • JWT Auth       │
│ • Tailwind CSS  │                  │ • SQLx/PostgreSQL│
│ • React Query   │                  │ • Tantivy Search │
│ • Zustand       │                  │ • Git Crawler    │
└─────────────────┘                  └──────────────────┘
                                               │
                                               ▼
                                     ┌──────────────────┐
                                     │   PostgreSQL     │
                                     │   (port 5432)    │
                                     └──────────────────┘
```

Happy coding! 🚀
