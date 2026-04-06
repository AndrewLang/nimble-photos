import { environment } from '../environments/environment';
export enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3,
    NONE = 4,
}

export class Logger {
    private level: LogLevel = LogLevel.DEBUG;

    constructor() {
        this.level = environment.production ? LogLevel.WARN : LogLevel.DEBUG;
    }

    setLevel(level: LogLevel) {
        this.level = level;
    }

    debug(...args: any[]) {
        if (this.level <= LogLevel.DEBUG) {
            this.write('debug', 'DEBUG', args);
        }
    }

    info(...args: any[]) {
        if (this.level <= LogLevel.INFO) {
            this.write('info', 'INFO', args);
        }
    }

    warn(...args: any[]) {
        if (this.level <= LogLevel.WARN) {
            this.write('warn', 'WARN', args);
        }
    }

    error(...args: any[]) {
        if (this.level <= LogLevel.ERROR) {
            this.write('error', 'ERROR', args);
        }
    }

    private write(method: 'debug' | 'info' | 'warn' | 'error', label: string, args: any[]) {
        const now = new Date().toISOString();
        if (method === 'error') {
            console.error(`[${label}] [${now}]`, ...args);
        } else if (method === 'warn') {
            console.warn(`[${label}] [${now}]`, ...args);
        } else {
            console.log(`[${label}] [${now}]`, ...args);
        }
    }
}

export const logger = new Logger();
