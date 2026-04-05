import { ConsoleHandler, getLogger, setup } from "$std/log/mod.ts";

setup({
  handlers: {
    default: new ConsoleHandler("DEBUG", {
      formatter: (record) => {
        const extra =
          record.args[0] != null && typeof record.args[0] === "object"
            ? (record.args[0] as Record<string, unknown>)
            : {};
        return JSON.stringify({
          timestamp: record.datetime.toISOString(),
          level: record.levelName.toLowerCase(),
          service: "soliloquio-client",
          event: record.msg,
          ...extra,
        });
      },
    }),
  },
  loggers: {
    default: { level: "DEBUG", handlers: ["default"] },
  },
});

type LogFields = {
  request_id?: string;
  message?: string;
  who?: { user_id?: string | null; ip?: string | null };
  where?: { path?: string; method?: string; status?: number };
  what?: { outcome?: string; reason?: string };
};

const _logger = getLogger();

export const logger = {
  info(event: string, fields: LogFields = {}) {
    _logger.info(event, fields);
  },
  warn(event: string, fields: LogFields = {}) {
    _logger.warn(event, fields);
  },
  error(event: string, fields: LogFields = {}) {
    _logger.error(event, fields);
  },
};
