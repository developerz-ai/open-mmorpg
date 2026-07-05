# Operator website: build the Solid SPA with Bun, serve the static bundle with nginx.
#   docker build -f docker/web.Dockerfile -t omm-web .
# syntax=docker/dockerfile:1
FROM oven/bun:1 AS builder
WORKDIR /build
COPY package.json bun.lock ./
COPY apps/web/package.json apps/web/package.json
COPY packages/i18n/package.json packages/i18n/package.json
COPY packages/ui/package.json packages/ui/package.json
RUN bun install --frozen-lockfile
COPY . .
RUN bun run --filter @omm/web build

FROM nginx:1.27-alpine AS runtime
COPY docker/web.nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /build/apps/web/dist /usr/share/nginx/html
EXPOSE 80
