# syntax=docker/dockerfile:1

# -----------------------------------------------------------------------------
# Build stage
# -----------------------------------------------------------------------------
FROM node:22-alpine AS builder

WORKDIR /app

COPY package*.json ./
# Native deps (utf-8-validate) have no prebuilt binaries for alpine/arm64 and
# fall back to node-gyp; the toolchain lives only in the builder stage.
RUN apk add --no-cache python3 make g++ && npm ci

# Vite inlines import.meta.env at BUILD time: for any non-local deployment
# these must be passed as build args (see docker-compose.yml `build.args` and
# DEPLOYMENT.md — VITE_API_URL should point at the public origin, e.g.
# https://<domaine>, served by the Caddy reverse proxy).
ARG VITE_API_URL=""
ARG VITE_WALLET_CONNECT_PROJECT_ID=""
ENV VITE_API_URL=$VITE_API_URL \
    VITE_WALLET_CONNECT_PROJECT_ID=$VITE_WALLET_CONNECT_PROJECT_ID

COPY . .
RUN npm run build

# -----------------------------------------------------------------------------
# Runtime stage
# -----------------------------------------------------------------------------
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
