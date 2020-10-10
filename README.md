# Zero2prod

## database connection
- we use sqlx that needs connection to database at compile time to figure out types, and find errors in queries (.env has config for it)

## tests
- init db ``./scripts/init_db.sh``
- ``cargo test``
