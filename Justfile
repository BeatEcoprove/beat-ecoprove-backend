set dotenv-load

default:
    @just --list

# Serve proxy localy
serve:
  cargo run

# Build microservice images
build-images:
  nix flake update
  nix run .#build-all -L

# Compose for development
compose-dev:
  $CONTAINER_TOOL compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# Compose for production
compose-build:
  $CONTAINER_TOOL compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Serve the application in development mode
dev: build-images compose-dev

# Serve the application in production mode
prod: build-images compose-build
