# Run `just` with no arguments to list the recipes.

db := "Plant-Watering.db"

# Server address for the HTTP recipes; override with `HOST=... just add ...`
host := env_var_or_default("HOST", "http://127.0.0.1:8080")

# List the available recipes
default:
    @just --list

# Start the server (creates the database and table if they are missing)
run:
    cargo run

# Insert the sample plants from seed.sql (safe to run more than once)
seed:
    @test -f {{db}} || (echo "{{db}} not found: run 'just run' once first to create it" && exit 1)
    sqlite3 {{db}} < seed.sql

# Delete the local database; the next `just run` recreates an empty one
reset:
    rm -f {{db}}

# Show every plant in the local database
plants:
    sqlite3 -header -column {{db}} "SELECT slug, name, interval_days, datetime(last_watered_at, 'unixepoch') AS last_watered FROM plants"

# --- HTTP: talk to the running server (start it with `just run`) -----------

# POST a never-watered plant, e.g. `just add pothos "Pothos (kitchen)" 5`
add slug name interval_days:
    curl -s -w '\nHTTP %{http_code}\n' -X POST {{host}}/api/plants \
        -H 'content-type: application/json' \
        -d '{"slug": "{{slug}}", "name": "{{name}}", "interval_days": {{interval_days}}, "last_watered_at": null}'

# POST a plant from a JSON file, e.g. `just post requests/monstera.json`
post file:
    curl -s -w '\nHTTP %{http_code}\n' -X POST {{host}}/api/plants \
        -H 'content-type: application/json' \
        -d @{{file}}

# GET one plant by slug (200 if it exists, 404 if not)
get slug:
    curl -s -w '\nHTTP %{http_code}\n' {{host}}/api/plants/{{slug}}

# GET the plant list
list:
    curl -s -w '\nHTTP %{http_code}\n' {{host}}/api/plants

# Simulate tapping a plant's NFC tag (waters it)
water slug:
    curl -s -w '\nHTTP %{http_code}\n' {{host}}/w/{{slug}}

# Run clippy and the tests
check:
    cargo clippy --all-targets
    cargo test
