#

## submodule
git submodule add https://github.com/AndrewLang/matrix-nimble-web crates/nimble-web

## Tag schema migration
Tag storage is normalized in `backend/migrations/20260206_normalize_tags.up.sql`:
- `tags` stores unique normalized tag names (`name_norm = lower(trim(name))`).
- `photo_tags` maps tags to photos.
- `album_tags` maps tags to albums.

The old `photos.tags` column is intentionally kept for compatibility, but new schema work should use the join tables.

Run migration manually with Postgres `psql`:

```bash
psql "$DATABASE_URL" -f backend/migrations/20260206_normalize_tags.up.sql
psql "$DATABASE_URL" -f backend/migrations/20260206_photos_public_visible.up.sql
```

Rollback:

```bash
psql "$DATABASE_URL" -f backend/migrations/20260206_photos_public_visible.down.sql
psql "$DATABASE_URL" -f backend/migrations/20260206_normalize_tags.down.sql
```
