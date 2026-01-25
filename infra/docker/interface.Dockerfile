# syntax=docker/dockerfile:1

FROM node:25-bookworm AS builder
WORKDIR /app

COPY console/interface/package.json console/interface/package-lock.json* console/interface/npm-shrinkwrap.json* ./console/interface/
RUN cd console/interface && (npm ci || npm install)

COPY console/interface ./console/interface
RUN cd console/interface && npm run build

FROM node:25-bookworm-slim AS runtime
WORKDIR /srv
ENV NODE_ENV=production
ENV PORT=7071

COPY --from=builder /app/console/interface/dist ./console/interface/dist
COPY --from=builder /app/console/interface/package.json ./console/interface/package.json
COPY --from=builder /app/console/interface/node_modules ./console/interface/node_modules

COPY infra/docker/runtime/entrypoint.sh /entrypoint.sh
COPY infra/docker/runtime/healthcheck.sh /healthcheck.sh
RUN chmod +x /entrypoint.sh /healthcheck.sh

EXPOSE 7071
HEALTHCHECK --interval=10s --timeout=3s --retries=10 CMD ["/healthcheck.sh","http://127.0.0.1:7071/healthz"]
ENTRYPOINT ["/entrypoint.sh"]
CMD ["node","console/interface/dist/index.js"]
