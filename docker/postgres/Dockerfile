FROM docker.io/postgres:17.6-bookworm

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        postgresql-17-postgis-3; \
    rm -rf /var/lib/apt/lists/*
