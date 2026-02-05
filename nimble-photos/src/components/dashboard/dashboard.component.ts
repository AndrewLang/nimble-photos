
import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { first } from 'rxjs';

import { DashboardSetting, DashboardSystemSection } from '../../models/dashboard-settings.model';
import { SectionSummary } from '../../models/dashboard-section-summary.model';
import { SettingsService } from '../../services/settings.service';

@Component({
    selector: 'mtx-dashboard',
    standalone: true,
    imports: [FormsModule],
    templateUrl: './dashboard.component.html',
    host: {
        class: 'flex-1 flex flex-col min-h-0 overflow-hidden',
    },
})
export class DashboardComponent implements OnInit {
    readonly settings = signal<DashboardSetting[]>([]);
    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly selectedSection = signal<DashboardSystemSection>('general');
    readonly localValues = signal<Record<string, unknown>>({});
    readonly saving = signal<Record<string, boolean>>({});
    readonly saveSuccess = signal<Record<string, boolean>>({});
    readonly fieldErrors = signal<Record<string, string>>({});

    private readonly settingsService = inject(SettingsService);

    readonly navSections = computed<SectionSummary[]>(() => {
        const aggregate: Record<DashboardSystemSection, SectionSummary> = {} as Record<
            DashboardSystemSection,
            SectionSummary
        >;
        for (const setting of this.settings()) {
            if (aggregate[setting.section]) {
                aggregate[setting.section].count += 1;
            } else {
                aggregate[setting.section] = {
                    section: setting.section,
                    label: setting.sectionLabel,
                    count: 1,
                };
            }
        }
        return Object.values(aggregate);
    });

    readonly sectionSettings = computed(() => {
        const section = this.selectedSection();
        return this.settings().filter(setting => setting.section === section);
    });

    getCurrentSectionLabel(): string {
        const summary = this.navSections().find(section => section.section === this.selectedSection());
        return summary?.label ?? '';
    }

    getSectionIcon(section: DashboardSystemSection): string {
        switch (section) {
            case 'general':
                return 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6';
            case 'experience':
                return 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z';
            case 'notifications':
                return 'M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9';
            case 'photo-manage':
                return 'M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z';
            default:
                return 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z';
        }
    }

    ngOnInit(): void {
        this.loadSettings();
    }

    private loadSettings(): void {
        this.loading.set(true);
        this.error.set(null);
        this.settingsService
            .getSettings()
            .pipe(first())
            .subscribe({
                next: result => {
                    this.settings.set(result);
                    this.buildLocalValues(result);
                    const firstSection = this.navSections()[0];
                    if (firstSection) {
                        this.selectedSection.set(firstSection.section);
                    }
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

    selectSection(section: DashboardSystemSection): void {
        this.selectedSection.set(section);
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
