import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class LocalSettingService {

    constructor() { }

    get<T>(key: string, defaultValue: T): T {
        const stored = localStorage.getItem(key);
        if (stored === null || !stored || stored === 'undefined') {
            return defaultValue;
        }
        try {
            let value = JSON.parse(stored) as T ?? defaultValue;
            console.debug(`LocalSettingService: Retrieved key "${key}" with value:`, value);
            return value;
        } catch {
            return defaultValue;
        }
    }

    set<T>(key: string, value: T): void {
        console.debug(`LocalSettingService: Setting key "${key}" to value:`, value);
        localStorage.setItem(key, JSON.stringify(value));
    }

    remove(key: string): void {
        localStorage.removeItem(key);
    }

    clear(): void {
        localStorage.clear();
    }
}