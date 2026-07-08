# syntax=docker/dockerfile:1

FROM node:20-alpine AS builder

WORKDIR /app

# Install build tooling required for native Node modules (e.g. bufferutil).
RUN apk add --no-cache python3 make g++

COPY package*.json ./
RUN npm ci

COPY . .
ARG VITE_API_URL=http://localhost:3001
ENV VITE_API_URL=$VITE_API_URL
RUN npm run build

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 3000
