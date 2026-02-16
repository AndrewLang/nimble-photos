import { Injectable, signal } from '@angular/core';
import { first } from 'rxjs';

import { DashboardSetting, DashboardSystemSection } from '../models/dashboard.settings.model';
import { SettingsService } from './settings.service';

@Injectable({
    providedIn: 'root',
})
export class DashboardSettingsService {
    readonly settings = signal<DashboardSetting[]>([]);
    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly localValues = signal<Record<string, unknown>>({});
    readonly saving = signal<Record<string, boolean>>({});
    readonly saveSuccess = signal<Record<string, boolean>>({});
    readonly fieldErrors = signal<Record<string, string>>({});

    private hasLoaded = false;

    constructor(private readonly settingsService: SettingsService) { }

    ensureLoaded(): void {
        if (this.loading() || this.hasLoaded) {
            return;
        }
        this.loadSettings();
    }

    getSectionSettings(section: DashboardSystemSection): DashboardSetting[] {
        return this.safeSettings().filter(setting => setting.section === section && setting.key !== 'storage.locations');
    }

    getSectionLabel(section: DashboardSystemSection): string {
        const match = this.safeSettings().find(setting => setting.section === section);
        return match?.sectionLabel ?? '';
    }

    getSectionCount(section: DashboardSystemSection): number {
        return this.safeSettings().filter(setting => setting.section === section).length;
    }

    getSettingByName(key: string): DashboardSetting | undefined {
        return this.safeSettings().find(setting => setting.key === key);
    }

    getSettingValue(key: string): unknown {
        return this.getSettingByName(key)?.value;
    }

    getLocalValue(key: string): unknown {
        return this.localValues()[key];
    }

    setLocalValue(key: string, value: unknown): void {
        this.localValues.update(current => ({ ...current, [key]: value }));
    }

    toggleBoolean(setting: DashboardSetting): void {
        const current = Boolean(this.getLocalValue(setting.key) ?? setting.value);
        this.setLocalValue(setting.key, !current);
    }

    saveSetting(setting: DashboardSetting): void {
        const normalized = this.normalizeValue(setting, this.getLocalValue(setting.key));
        if (!normalized.ok) {
            this.fieldErrors.update(current => ({
                ...current,
                [setting.key]: normalized.message || 'Invalid value',
            }));
            return;
        }

        this.fieldErrors.update(current => ({ ...current, [setting.key]: '' }));
        this.saving.update(current => ({ ...current, [setting.key]: true }));

        this.settingsService
            .updateSetting(setting.key, normalized.value)
            .pipe(first())
            .subscribe({
                next: updated => {
                    this.settings.update(current => current.map(s => (s.key === updated.key ? updated : s)));
                    const storedValue =
                        updated.valueType === 'json'
                            ? JSON.stringify(updated.value ?? {}, null, 2)
                            : updated.value;
                    this.localValues.update(current => ({ ...current, [updated.key]: storedValue }));
                    this.saveSuccess.update(current => ({ ...current, [updated.key]: true }));
                    window.setTimeout(() => {
                        this.saveSuccess.update(current => ({ ...current, [updated.key]: false }));
                    }, 2000);
                },
                error: err => {
                    this.fieldErrors.update(current => ({
                        ...current,
                        [setting.key]: err.message || 'Failed to save setting.',
                    }));
                },
                complete: () => {
                    this.saving.update(current => ({ ...current, [setting.key]: false }));
                },
            });
    }

    isSettingEnabled(setting: DashboardSetting): boolean {
        return Boolean(this.getLocalValue(setting.key));
    }

    private loadSettings(): void {
        this.loading.set(true);
        this.error.set(null);
        this.settingsService
            .getSettings()
            .pipe(first())
            .subscribe({
                next: result => {
                    const normalized = Array.isArray(result) ? result : [];
                    this.settings.set(normalized);
                    this.buildLocalValues(normalized);
                    this.hasLoaded = true;
                },
                error: err => {
                    this.error.set(err.message || 'Failed to load dashboard settings.');
                },
                complete: () => {
                    this.loading.set(false);
                },
            });
    }

    private buildLocalValues(settings: DashboardSetting[]): void {
        const snapshot: Record<string, unknown> = {};
        for (const setting of settings) {
            if (setting.valueType === 'json') {
                snapshot[setting.key] = JSON.stringify(setting.value ?? {}, null, 2);
            } else {
                snapshot[setting.key] = setting.value;
            }
        }

        this.localValues.set(snapshot);
    }

    private safeSettings(): DashboardSetting[] {
        const current = this.settings();
        return Array.isArray(current) ? current : [];
    }

    private normalizeValue(
        setting: DashboardSetting,
        rawValue: unknown,
    ): { ok: true; value: unknown } | { ok: false; message?: string } {
        switch (setting.valueType) {
            case 'number':
                const numeric = Number(rawValue);
                if (Number.isNaN(numeric)) {
                    return { ok: false, message: 'Please enter a valid number.' };
                }
                return { ok: true, value: numeric };
            case 'boolean':
                return { ok: true, value: Boolean(rawValue) };
            case 'json':
                if (typeof rawValue !== 'string') {
                    return { ok: true, value: rawValue };
                }
                const trimmed = rawValue.trim();
                if (!trimmed) {
                    return { ok: true, value: {} };
                }
                try {
                    return { ok: true, value: JSON.parse(trimmed) };
                } catch {
                    return { ok: false, message: 'JSON must be well formed.' };
                }
            default:
                return { ok: true, value: rawValue ?? '' };
        }
    }
}
