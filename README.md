# Nimble Photos

Nimble Photos is a self-hosted photo management application with an Angular frontend and a Rust backend.
It is designed for personal or small-team use, with a focus on fast browsing, album organization, and secure access control.

## Features

- Photo gallery with thumbnail and preview support
- Album management
- Authentication and role-based access
- Public/private site controls and upload permissions
- Dashboard settings management
- PostgreSQL-backed data storage
- Single-service hosting for frontend and backend (backend serves built frontend files)

## Docker Modes

### Deploy (uses prebuilt image)

### 1. Configure environment

Copy `.env.example` to `.env` and update values as needed:

- `POSTGRES_DATA_DIR`: host folder for PostgreSQL data
- `APP_DATA_DIR`: host folder for app data used by Nimble Photos
- `APP_PORT`: host port for Nimble Photos (default `5151`)
- `POSTGRES_PORT`: host port for PostgreSQL (default `5438`)
- `APP_IMAGE`: deploy image tag (default `nimble-photos:latest`)

### 2. Build and start services

From the repository root:

```bash
docker compose -f docker-compose.deploy.yml up -d
```

This starts:

- `postgres` (database)
- `app` (Nimble Photos backend + served frontend)

### 3. Access the app

Open:

`http://localhost:5151`

If you changed `APP_PORT`, use that port instead.

### Local Build (builds Dockerfile)

```bash
docker compose up --build -d
```

### Dev DB only (separate from deploy image)

```bash
docker compose --env-file .env.dev -f docker-compose.dev.yml up -d

docker compose --env-file .env.dev -f docker-compose.dev.yml down

```

Clean db
```Sql
DELETE FROM public.photos;
DELETE FROM public.exifs;
```


```deploy
docker compose -f docker-compose.deploy.yml up -d
```