/**
 * Centralized logging utility
 * Provides consistent logging with environment-aware behavior
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LoggerConfig {
  enabled: boolean;
  level: LogLevel;
  prefix?: string;
}

const isDevelopment = import.meta.env.DEV;

const defaultConfig: LoggerConfig = {
  enabled: isDevelopment,
  level: 'debug',
};

class Logger {
  private config: LoggerConfig;

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
  }

  private shouldLog(level: LogLevel): boolean {
    if (!this.config.enabled) return false;

    const levels: LogLevel[] = ['debug', 'info', 'warn', 'error'];
    const currentLevelIndex = levels.indexOf(this.config.level);
    const messageLevelIndex = levels.indexOf(level);

    return messageLevelIndex >= currentLevelIndex;
  }

  private formatMessage(level: LogLevel, message: string, ...args: any[]): string {
    const timestamp = new Date().toISOString();
    const prefix = this.config.prefix ? `[${this.config.prefix}]` : '';
    return `${timestamp} ${prefix}[${level.toUpperCase()}] ${message}`;
  }

  debug(message: string, ...args: any[]): void {
    if (this.shouldLog('debug')) {
      console.log(this.formatMessage('debug', message), ...args);
    }
  }

  info(message: string, ...args: any[]): void {
    if (this.shouldLog('info')) {
      console.info(this.formatMessage('info', message), ...args);
    }
  }

  warn(message: string, ...args: any[]): void {
    if (this.shouldLog('warn')) {
      console.warn(this.formatMessage('warn', message), ...args);
    }
  }

  error(message: string, ...args: any[]): void {
    if (this.shouldLog('error')) {
      console.error(this.formatMessage('error', message), ...args);
    }
  }

  performance(componentName: string, renderTime: number): void {
    if (this.shouldLog('debug')) {
      console.log(
        this.formatMessage('debug', `Performance: ${componentName} rendered in ${renderTime.toFixed(2)}ms`)
      );
    }
  }
}

// Export singleton instances for different modules
export const logger = new Logger();
export const storeLogger = new Logger({ prefix: 'Store' });
export const hookLogger = new Logger({ prefix: 'Hook' });
export const apiLogger = new Logger({ prefix: 'API' });

export default logger;
