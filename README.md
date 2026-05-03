# Rust Backend Starter

Production-ready Rust backend with **Axum + PostgreSQL + JWT Auth**, fully containerized.

## Stack

| Layer | Crate |
|---|---|
| Web framework | `axum` |
| Database | `sqlx` + PostgreSQL |
| Auth | `jsonwebtoken` (HS256) |
| Password hashing | `argon2` (Argon2id) |
| Validation | `validator` |
| Config | `dotenvy` |
| Logging | `tracing` + `tracing-subscriber` |
| CORS | `tower-http` |

---

## Quick Start (Docker)

Projekt jest zoptymalizowany pod pracę z Dockerem. Nie musisz instalować Rusta ani PostgreSQL lokalnie.

### 1. Konfiguracja środowiska
Skopiuj przykład i ustaw swoje zmienne (szczególnie `JWT_SECRET` oraz `ADMIN_PASSWORD`):
```bash
cp .env.example .env
```

### 2. Uruchomienie projektu
```bash
docker compose up --build
```

Serwer będzie dostępny pod adresem: `http://localhost:3000`
Interaktywna dokumentacja API: `http://localhost:3000/docs`

---

## Workflow Deweloperski

### Zmiany w kodzie
```bash
docker compose up --build
```

### Resetowanie bazy danych
```bash
docker compose down -v
docker compose up --build
```

---

## Testowanie

Najlepiej uruchamiać testy wewnątrz kontenera:
```bash
docker compose run app cargo test
```

---

## API Reference

### POST /auth/login
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "twoje_haslo"}'
```

### GET /dashboard (JWT)
```bash
curl http://localhost:3000/dashboard -H "Authorization: Bearer <TOKEN>"
```

---

## Struktura Projektu

- `src/lib.rs`: Główna logika i definicja routera.
- `src/main.rs`: Entrypoint serwera.
- `src/routes/`: Handlery API.
- `src/middleware/`: Auth i zabezpieczenia.
- `src/db.rs`: Baza danych.
- `tests/`: Testy integracyjne.
