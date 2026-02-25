import { Component, computed, inject, signal } from '@angular/core';

import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { PhotoUploadComponent } from './photo.upload.component';
import { PhotoVisibilityComponent } from './photo.visibility.component';

const PhotoManageTabs = {
    Upload: 'upload',
    Visibility: 'visibility',
} as const;
type PhotoManageTab = typeof PhotoManageTabs[keyof typeof PhotoManageTabs];
@Component({
    selector: 'mtx-photo-manage-setting',
    templateUrl: './photo.manage.setting.component.html',
    imports: [PhotoUploadComponent, PhotoVisibilityComponent],
})
export class PhotoManageSettingComponent {
    private readonly settingsService = inject(DashboardSettingsService);

    readonly activeTab = signal<PhotoManageTab>(PhotoManageTabs.Upload);
    readonly tabs = computed(() => [
        { key: PhotoManageTabs.Upload, label: 'Upload' },
        { key: PhotoManageTabs.Visibility, label: 'Visibility' },
    ]);

    setActiveTab(tab: PhotoManageTab): void {
        this.activeTab.set(tab);
    }

    constructor() {
        this.settingsService.ensureLoaded();
    }
}
