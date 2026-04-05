FROM denoland/deno:2.7.11
WORKDIR /app
COPY . .
RUN deno task build
CMD ["deno", "run", "-A", "main.ts"]
