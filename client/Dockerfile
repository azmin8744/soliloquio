FROM denoland/deno:2.9.8
WORKDIR /app
COPY . .
RUN deno task build
CMD ["deno", "run", "-A", "main.ts"]
