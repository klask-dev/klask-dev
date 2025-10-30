# [![klask.dev](https://raw.githubusercontent.com/klask-dev/klask-dev/refs/heads/master/resources/images/klask-dev-clean-v2.svg)](https://github.com/klask-dev/klask-dev)

| Branch  | Build  | Coverage  |
|---|---|---|
| master  | [![CI/CD Pipeline](https://github.com/klask-dev/klask-dev/actions/workflows/ci.yml/badge.svg)](https://github.com/klask-dev/klask-dev/actions/workflows/ci.yml)  | [![Coverage Status](https://img.shields.io/coveralls/klask-dev/klask-dev/master.svg?style=flat-square)](https://coveralls.io/github/klask-dev/klask-dev?branch=master) |

#### Docker
[![Docker Stars](https://img.shields.io/docker/stars/klask/klask.dev.svg?style=flat-square)](https://hub.docker.com/r/klask/klask.dev/) [![Docker pulls](https://img.shields.io/docker/pulls/klask/klask.dev.svg?style=flat-square)](https://hub.docker.com/r/klask/klask.dev/) [![Docker build](https://img.shields.io/docker/automated/klask/klask.dev.svg?style=flat-square)](https://hub.docker.com/r/klask/klask.dev/builds/)

## What is Klask?

**Klask** is a modern, high-performance search engine for source code. Built with Rust and React, it provides fast, accurate code search across multiple Git repositories with advanced filtering and syntax highlighting.

## 🚀 Modern Architecture (v2.x)

The latest version of Klask uses cutting-edge technologies for performance and developer experience:

### Backend
- **Rust** - High-performance, memory-safe systems programming
- **Axum** - Modern async web framework
- **Tantivy** - Full-text search engine (Rust equivalent of Lucene)
- **PostgreSQL** - Robust relational database
- **SQLx** - Compile-time SQL verification

### Frontend
- **React 18** - Modern UI library with concurrent features
- **TypeScript** - Type-safe JavaScript
- **Vite** - Lightning-fast build tool
- **TailwindCSS** - Utility-first CSS framework
- **React Query** - Powerful data fetching and caching

### Features
- ✅ Multi-repository indexing (Git, GitLab, GitHub)
- ✅ Real-time full-text search with Tantivy
- ✅ JWT-based authentication
- ✅ Syntax highlighting for 100+ languages
- ✅ Advanced filtering (branches, projects, file types)
- ✅ Scheduled auto-crawling with cron
- ✅ Admin dashboard with metrics
- ✅ Docker and Kubernetes ready

---

## 📦 Quick Start

### Local Development (Docker Compose)

The fastest way to run Klask locally:

```bash
# Clone the repository
git clone https://github.com/klask-dev/klask-dev.git
cd klask-dev

# Start all services with docker-compose
docker-compose up -d

# Access Klask at http://localhost:5173
# Backend API at http://localhost:3000
```

See [`docker-compose.yml`](docker-compose.yml) for full configuration.

### Manual Development Setup

#### Prerequisites
- **Rust** 1.70+ ([install via rustup](https://rustup.rs/))
- **Node.js** 18+ and npm
- **PostgreSQL** 14+
- **Docker** (optional, for database)

#### 1. Start PostgreSQL Database

```bash
# Using Docker (recommended)
docker-compose -f docker-compose.dev.yml up -d

# Or use your own PostgreSQL instance
# Make sure to create a database named 'klask'
```

#### 2. Start Backend (Rust)

```bash
cd klask-rs

# Install dependencies and run migrations
cargo build
sqlx migrate run

# Start the backend server
cargo run --bin klask-rs

# Backend will be available at http://localhost:3000
```

#### 3. Start Frontend (React)

```bash
cd klask-react

# Install dependencies
npm install

# Start the development server
npm run dev

# Frontend will be available at http://localhost:5173
```

---

## 🐳 Docker Deployment

### Docker Compose (Production)

```bash
# Pull and start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Docker Images

```bash
# Backend
docker pull klask/klask-backend:latest
docker run -p 3000:3000 -e DATABASE_URL=postgres://... klask/klask-backend:latest

# Frontend
docker pull klask/klask-frontend:latest
docker run -p 5173:80 klask/klask-frontend:latest
```

---

## ☸️ Kubernetes Deployment (Helm)

### Quick Install

```bash
# Add Klask Helm repository (if available)
helm repo add klask https://charts.klask.dev
helm repo update

# Install with default values
helm install klask klask/klask

# Install with custom values
helm install klask klask/klask -f my-values.yaml
```

### Basic Configuration Example

Create a `values.yaml`:

```yaml
backend:
  replicaCount: 2
  image:
    repository: klask/klask-backend
    tag: "2.1.0"

  resources:
    requests:
      memory: "512Mi"
      cpu: "500m"
    limits:
      memory: "1Gi"
      cpu: "1000m"

frontend:
  replicaCount: 2
  image:
    repository: klask/klask-frontend
    tag: "2.1.0"

postgresql:
  enabled: true
  auth:
    username: klask
    password: changeme
    database: klask

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: klask.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: klask-tls
      hosts:
        - klask.example.com
```

Then install:

```bash
helm install klask ./charts/klask -f values.yaml
```

See full Helm documentation: [charts/klask/README.md](charts/klask/README.md)

---

## 🛠️ Development

### Project Structure

```
klask-dev/
├── klask-rs/           # Rust backend (Axum + Tantivy + PostgreSQL)
│   ├── src/
│   │   ├── api/        # REST API endpoints
│   │   ├── models/     # Database models
│   │   ├── services/   # Business logic (crawler, search, etc.)
│   │   └── repositories/ # Database queries
│   ├── migrations/     # SQL migrations (SQLx)
│   └── Cargo.toml
│
├── klask-react/        # React frontend (TypeScript + Vite + TailwindCSS)
│   ├── src/
│   │   ├── api/        # API client (React Query)
│   │   ├── components/ # Reusable UI components
│   │   ├── features/   # Feature modules (search, admin, etc.)
│   │   └── hooks/      # Custom React hooks
│   └── package.json
│
├── charts/klask/       # Helm chart for Kubernetes deployment
├── .claude/            # AI-assisted development tools
└── docker-compose.yml  # Local development environment
```

### Running Tests

```bash
# Backend tests (Rust)
cd klask-rs
cargo test

# Frontend tests (React)
cd klask-react
npm test

# Integration tests
npm run test:integration
```

### Building for Production

```bash
# Backend
cd klask-rs
cargo build --release

# Frontend
cd klask-react
npm run build

# Docker images
docker build -t klask/klask-backend:latest -f klask-rs/Dockerfile .
docker build -t klask/klask-frontend:latest -f klask-react/Dockerfile .
```

---

## 📚 Documentation

- **[Development Guide](CLAUDE.md)** - Comprehensive guide for contributors
- **[Helm Deployment](charts/klask/README.md)** - Kubernetes deployment guide
- **[API Documentation](docs/API.md)** - REST API reference
- **[Architecture](docs/ARCHITECTURE.md)** - System architecture overview

---

## 🤝 Contributing

Contributions are welcome! Please read our [contributing guide](CONTRIBUTING.md) before submitting pull requests.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test && npm test`)
5. Commit your changes (`git commit -m 'feat: add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

---

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

## 🔗 Links

- **Website**: [klask.dev](https://klask.dev)
- **Documentation**: [docs.klask.dev](https://docs.klask.dev)
- **Issue Tracker**: [GitHub Issues](https://github.com/klask-dev/klask-dev/issues)
- **Discussions**: [GitHub Discussions](https://github.com/klask-dev/klask-dev/discussions)

---

## 🙏 Acknowledgments

Built with powerful open-source technologies:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tantivy](https://github.com/tantivy-search/tantivy) - Full-text search engine
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [React](https://react.dev/) - UI library
- [Vite](https://vitejs.dev/) - Build tool
- [TailwindCSS](https://tailwindcss.com/) - CSS framework
