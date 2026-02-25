import { Component, computed, effect, inject, signal } from '@angular/core';

import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { PhotoService } from '../../services/photo.service';
import { ActionSelectorComponent } from './action.selector.component';

@Component({
    selector: 'mtx-photo-visibility',
    templateUrl: 'photo.visibility.component.html',
    imports: [ActionSelectorComponent],
})
export class PhotoVisibilityComponent {
    private readonly settingsService = inject(DashboardSettingsService);
    private readonly photoService = inject(PhotoService);

    readonly availableTags = signal<string[]>([]);
    readonly viewerHiddenTags = signal<string[]>([]);
    readonly tagsLoading = signal(false);
    readonly tagVisibilityError = signal<string | null>(null);
    readonly savingTagVisibility = signal(false);
    readonly tagVisibilitySaved = signal(false);

    readonly tagOptions = computed(() => {
        const fromApi = this.availableTags();
        const selected = this.viewerHiddenTags();
        return Array.from(new Set([...fromApi, ...selected]))
            .sort((a, b) => a.localeCompare(b))
            .map(tag => ({ key: tag, label: tag }));
    });

    constructor() {
        this.settingsService.ensureLoaded();
        this.loadTags();

        effect(() => {
            const setting = this.settingsService.getSettingByName('photo.manage.viewerHiddenTags');
            const raw = setting?.value;
            const parsed = Array.isArray(raw) ? raw : [];
            const normalized = Array.from(
                new Set(parsed.filter(item => typeof item === 'string').map(item => item.trim()).filter(Boolean)),
            ).sort((a, b) => a.localeCompare(b));
            this.viewerHiddenTags.set(normalized);
        });
    }

    onViewerHiddenTagsChange(tags: string[]): void {
        const normalized = Array.from(new Set(tags.map(tag => tag.trim()).filter(tag => tag.length)))
            .sort((a, b) => a.localeCompare(b));
        this.viewerHiddenTags.set(normalized);
        this.tagVisibilityError.set(null);
        this.tagVisibilitySaved.set(false);
    }

    saveViewerHiddenTags(): void {
        const setting = this.settingsService.getSettingByName('photo.manage.viewerHiddenTags');
        if (!setting) {
            this.tagVisibilityError.set('Viewer hidden tags setting was not found.');
            return;
        }

        const payload = this.viewerHiddenTags();
        this.savingTagVisibility.set(true);
        this.tagVisibilityError.set(null);
        this.tagVisibilitySaved.set(false);

        this.settingsService.setLocalValue(setting.key, JSON.stringify(payload, null, 2));
        this.settingsService.saveSetting(setting);

        window.setTimeout(() => {
            this.savingTagVisibility.set(false);
            const fieldError = this.settingsService.fieldErrors()[setting.key];
            if (fieldError) {
                this.tagVisibilityError.set(fieldError);
                return;
            }
            this.tagVisibilitySaved.set(true);
            window.setTimeout(() => this.tagVisibilitySaved.set(false), 1800);
        }, 250);
    }

    private loadTags(): void {
        this.tagsLoading.set(true);
        this.photoService.getAllPhotoTags().subscribe({
            next: tags => {
                const normalized = Array.from(new Set(tags.map(tag => tag.trim()).filter(Boolean)))
                    .sort((a, b) => a.localeCompare(b));
                this.availableTags.set(normalized);
                this.tagsLoading.set(false);
            },
            error: () => {
                this.availableTags.set([]);
                this.tagsLoading.set(false);
            },
        });
    }
}
