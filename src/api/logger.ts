/**
 * Centralized logger for the frontend.
 * 
 * Replaces scattered console.log calls throughout the codebase.
 * In development, logs to console with [Frontend] prefix.
 * In production, can be routed through Tauri tracing.
 */

const isDev = process.env.NODE_ENV !== 'production';

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const LOG_PREFIX = '[Frontend]';

function log(level: LogLevel, message: string, data?: unknown): void {
  const timestamp = new Date().toISOString();
  
  if (!isDev && level === 'debug') {
    return; // Skip debug logs in production
  }

  switch (level) {
    case 'debug':
      console.debug(`${LOG_PREFIX} ${message}`, data ?? '');
      break;
    case 'info':
      console.info(`${LOG_PREFIX} ${message}`, data ?? '');
      break;
    case 'warn':
      console.warn(`${LOG_PREFIX} ${message}`, data ?? '');
      break;
    case 'error':
      console.error(`${LOG_PREFIX} ${message}`, data ?? '');
      break;
  }
}

export const logger = {
  debug: (message: string, data?: unknown) => log('debug', message, data),
  info: (message: string, data?: unknown) => log('info', message, data),
  warn: (message: string, data?: unknown) => log('warn', message, data),
  error: (message: string, data?: unknown) => log('error', message, data),
};
