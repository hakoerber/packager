db="$(mktemp)"

export DATABASE_URL="sqlite://${db}"

cargo sqlx database create
cargo sqlx migrate run
cargo sqlx prepare
