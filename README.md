# Rust Backend Starter

Production-ready Rust backend with **Axum + PostgreSQL + JWT Auth**, fully containerized with Docker.

## Tech Stack

| Layer | Crate |
|---|---|
| Web framework | `axum` |
| Database | `sqlx` + PostgreSQL |
| Documentation | `utoipa` + `swagger-ui` |
| Auth | `jsonwebtoken` (HS256) |
| Password hashing | `argon2` (Argon2id) |
| Validation | `validator` |
| Config | `dotenvy` |
| Logging | `tracing` + `tracing-subscriber` |
| CORS | `tower-http` |

---

## Quick Start (Docker)

The project is optimized for Docker. You don't need Rust or PostgreSQL installed locally.

### 1. Environment Configuration
Copy the example environment file and set your variables (especially `JWT_SECRET` and `ADMIN_PASSWORD`):
```bash
cp .env.example .env
```

### 2. Run the Project
```bash
docker compose up --build
```

- **API Server**: `http://localhost:3000`
- **Interactive API Docs (Swagger)**: `http://localhost:3000/docs`

---

## Development Workflow

### Rebuilding after code changes
To apply changes made in `.rs` files, rebuild the container:
```bash
docker compose up --build
```

### Resetting the Database
To clear all data (e.g., after changing the admin password or SQL migrations):
```bash
docker compose down -v
docker compose up --build
```
*The `-v` flag removes volumes, including the physical PostgreSQL data files.*

---

## Testing

It is recommended to run tests inside the container to ensure they have access to the database:

```bash
docker compose run --rm test
```

### Test Structure
- `tests/api_tests.rs`: Integration tests for the full router using `tower::oneshot` (no network sockets required).
- Unit tests can be added directly within the `src/` modules.

---

## API Reference

### Health & Info
- `GET /`: Simple welcome message.
- `GET /health`: System health status (JSON).

### Authentication
- `POST /auth/register`: Register a new user.
- `POST /auth/login`: Login and receive access/refresh tokens.
- `POST /auth/refresh`: Refresh your access token.

### Protected Endpoints
- `GET /dashboard`: User dashboard (Requires JWT).
- `GET /admin/users`: List all users (Requires Admin JWT).

---

## Project Structure

- `src/lib.rs`: Main logic and router definition (used by `main.rs` and tests).
- `src/main.rs`: Server entry point.
- `src/routes/`: API handlers grouped by module.
- `src/middleware/`: Custom middleware (Auth, etc.).
- `src/db.rs`: Database connection and migrations.
- `tests/`: Integration tests.
