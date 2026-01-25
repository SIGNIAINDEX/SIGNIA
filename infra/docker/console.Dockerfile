# syntax=docker/dockerfile:1

FROM node:25-bookworm AS builder
WORKDIR /app

COPY console/web/package.json console/web/package-lock.json* console/web/npm-shrinkwrap.json* ./console/web/
RUN cd console/web && (npm ci || npm install)

COPY console/web ./console/web
RUN cd console/web && npm run build

FROM node:25-bookworm-slim AS runtime
WORKDIR /srv
ENV NODE_ENV=production
ENV PORT=3000

COPY --from=builder /app/console/web/.next ./console/web/.next
COPY --from=builder /app/console/web/public ./console/web/public
COPY --from=builder /app/console/web/package.json ./console/web/package.json
COPY --from=builder /app/console/web/node_modules ./console/web/node_modules
COPY --from=builder /app/console/web/next.config.js ./console/web/next.config.js

COPY infra/docker/runtime/entrypoint.sh /entrypoint.sh
COPY infra/docker/runtime/healthcheck.sh /healthcheck.sh
RUN chmod +x /entrypoint.sh /healthcheck.sh

EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=3s --retries=10 CMD ["/healthcheck.sh","http://127.0.0.1:3000/"]
ENTRYPOINT ["/entrypoint.sh"]
CMD ["node","console/web/node_modules/next/dist/bin/next","start","-p","3000","-H","0.0.0.0","--dir","console/web"]
