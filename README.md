# Task Board Project

A full-stack task board application with:

- **Frontend**: Flutter  
- **Backend**: Rust  
- **Database**: SQLite  
- **Infrastructure**: Docker, CI/CD pipelines, Kubernetes deployment  



---

### Development Notes

This project uses [SQLx](https://github.com/launchbadge/sqlx) in offline mode.

- All migrations live in `backend/migrations/`.
- At runtime, the backend automatically applies migrations to the DB.
- SQLx offline data is cached in `backend/.sqlx/` and committed to the repo, so `cargo build` works without a running database.

If you add or change queries:
1. Start a test database (`sqlite://dev.db`).
2. Run migrations (`sqlx migrate run`).
3. Run `cargo sqlx prepare -- --lib`.
4. Commit the updated `.sqlx/` folder.
