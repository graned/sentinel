# ── Stage 1: build sentinel-auth-sdk ─────────────────────────────────────────
FROM node:22-alpine AS sdk-builder

WORKDIR /packages/sentinel-auth-sdk
COPY packages/sentinel-auth-sdk/package.json ./
RUN npm install && npm cache clean --force
COPY packages/sentinel-auth-sdk/ ./
RUN npm run build

# ── Stage 2: build sentinel-auth-react ───────────────────────────────────────
FROM node:22-alpine AS auth-react-builder

# Provide the compiled SDK at the path the file: dependency resolves to.
WORKDIR /packages/sentinel-auth-sdk
COPY packages/sentinel-auth-sdk/package.json ./
COPY --from=sdk-builder /packages/sentinel-auth-sdk/dist ./dist

WORKDIR /packages/sentinel-auth-react
COPY packages/sentinel-auth-react/package.json ./
RUN npm install --legacy-peer-deps && npm cache clean --force
COPY packages/sentinel-auth-react/ ./
RUN npm run build

# ── Stage 3: build sentinel-ui ────────────────────────────────────────────────
FROM node:22-alpine AS ui-builder

# VITE_API_URL is baked into the JS bundle at build time.
# Pass it via --build-arg when running `docker build` or via the compose
# build.args section.  Defaults to an empty string which causes the SDK
# client to fall back to a relative URL (useful behind a reverse proxy that
# forwards /v1 to the backend on the same origin).
ARG VITE_API_URL=""
ENV VITE_API_URL=$VITE_API_URL

# Make the compiled local packages available.
WORKDIR /packages/sentinel-auth-sdk
COPY packages/sentinel-auth-sdk/package.json ./
COPY --from=sdk-builder /packages/sentinel-auth-sdk/dist ./dist

WORKDIR /packages/sentinel-auth-react
COPY packages/sentinel-auth-react/package.json ./
COPY --from=auth-react-builder /packages/sentinel-auth-react/dist ./dist

WORKDIR /app
COPY apps/sentinel-ui/package.json ./
RUN npm install --legacy-peer-deps && npm cache clean --force

COPY apps/sentinel-ui/ ./
RUN npm run build

# ── Stage 4: serve with nginx ─────────────────────────────────────────────────
FROM nginx:1.27-alpine

# Remove the default nginx config.
RUN rm /etc/nginx/conf.d/default.conf

COPY apps/sentinel-ui/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=ui-builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
