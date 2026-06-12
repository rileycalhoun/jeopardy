#!/bin/sh
set -eu

migrations_dir=/app/migrations
sqlx_bin=${SQLX_BIN:-sqlx}

ledger_exists=$(
    psql "$DATABASE_URL" -Atqc \
        "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL"
)

if [ "$ledger_exists" = "f" ]; then
    legacy_schema_complete=$(
        psql "$DATABASE_URL" -Atqc "
            SELECT
                to_regclass('public.games') IS NOT NULL
                AND to_regclass('public.players') IS NOT NULL
                AND to_regclass('public.admin_tokens') IS NOT NULL
                AND EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = 'games'
                      AND column_name = 'status'
                )
                AND EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = 'games'
                      AND column_name = 'question_pack_id'
                )
                AND EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = 'games'
                      AND column_name = 'started_at'
                )
                AND EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = 'games'
                      AND column_name = 'completed_at'
                )
        "
    )

    if [ "$legacy_schema_complete" = "t" ]; then
        echo "Existing schema detected without SQLx migration history; recording baseline."

        baseline_sql=$(mktemp)
        trap 'rm -f "$baseline_sql"' EXIT

        cat >"$baseline_sql" <<'SQL'
BEGIN;

CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
);
SQL

        for migration in "$migrations_dir"/*.sql; do
            filename=$(basename "$migration" .sql)
            version=${filename%%_*}
            case "$version" in
                *[!0-9]*)
                    echo "Invalid migration version in $filename" >&2
                    exit 1
                    ;;
            esac
            description=${filename#*_}
            description=$(printf '%s' "$description" | tr '_' ' ')
            description=$(printf '%s' "$description" | sed "s/'/''/g")
            checksum=$(openssl dgst -sha384 "$migration" | awk '{print $2}')

            cat >>"$baseline_sql" <<SQL
INSERT INTO _sqlx_migrations (
    version,
    description,
    success,
    checksum,
    execution_time
)
VALUES (
    $version,
    '$description',
    true,
    decode('$checksum', 'hex'),
    0
);
SQL
        done

        printf '\nCOMMIT;\n' >>"$baseline_sql"
        psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f "$baseline_sql"
        rm -f "$baseline_sql"
        trap - EXIT
    fi
fi

exec "$sqlx_bin" migrate run --source "$migrations_dir"
