# Arguments
image_name := "backend"
image_tag := "latest"
image_archive := "backend.tar"
release := "true"
start_app := "true"


# Variables
cargo_args := if release == "true" { "--release" } else { "" }
cargo_mode := if release == "true" { "release" } else { "debug" }
compose_app_file := if start_app == "true" { "-f docker-compose.application.yml" } else { "" }


# Build
#
# Builds the application.
#
# Args:
#   release: "true" or "false"
build:
    cargo build {{cargo_args}}


# Test
#
# Performs project unit tests.
test_unit:
    cargo test


# Lint
#
# Performs project linting.
lint:
    cargo clippy -- -D warnings


# Format
#
# Performs project formatting.
fmt:
    cargo +nightly fmt


# Doc
#
# Generates project documentation.
doc:
    cargo doc --workspace --all-features --no-deps --document-private-items --open


# Run
#
# Runs the application.
#
# Args:
#   release: "true" or "false"
run:
    cargo run {{cargo_args}}


# Image build
#
# Builds the Docker image.
#
# Args:
#   image_name: Name of the Docker image to build
#   image_tag: Tag of the Docker image to build
#   release: "true" or "false"
image_build:
    docker build -t {{image_name}}:{{image_tag}} \
                 --build-arg BUILD_MODE={{cargo_mode}} \
                 --build-arg BUILD_ARGS={{cargo_args}} \
                 .


# Image save
#
# Exports the Docker image to a tar file.
#
# Args:
#   image_name: Name of the Docker image to export
#   image_tag: Tag of the Docker image to export
#   output: Path to the output tar file
image_save:
    docker save -o {{image_archive}} {{image_name}}:{{image_tag}}


# Image load
#
# Imports the Docker image from a tar file.
#
# Args:
#   input: Path to the input tar file
image_load:
    docker load -i {{image_archive}}


# Down
#
# Stops the local enviroment.
down:
    docker-compose -f docker-compose.yml \
                   -f docker-compose.application.yml \
                   down


# Up
#
# Starts the local enviroment.
#
# Args:
#   start_app: "true" or "false"
up: down
    docker-compose -f docker-compose.yml \
                   {{compose_app_file}} \
                   up --abort-on-container-exit