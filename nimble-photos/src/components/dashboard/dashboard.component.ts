
import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { first } from 'rxjs';

import { DashboardSetting, DashboardSystemSection } from '../../models/dashboard-settings.model';
import { SectionSummary } from '../../models/dashboard-section-summary.model';
import { SettingsService } from '../../services/settings.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-dashboard',
    standalone: true,
    imports: [FormsModule, SvgComponent],
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
                return 'sectionGeneral';
            case 'experience':
                return 'sectionExperience';
            case 'notifications':
                return 'sectionNotifications';
            case 'photo-manage':
                return 'sectionPhotoManage';
            default:
                return 'sectionDefault';
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
