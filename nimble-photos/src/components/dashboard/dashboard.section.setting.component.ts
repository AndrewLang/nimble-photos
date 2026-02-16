import { Component, Input, OnInit, computed, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { DashboardSetting, DashboardSystemSection } from '../../models/dashboard.settings.model';
import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { SvgComponent } from '../svg/svg.component';
import { LogoEditorComponent } from '../shared/logo-editor/logo.editor.component';

@Component({
    selector: 'mtx-dashboard-section-setting',
    imports: [FormsModule, SvgComponent, LogoEditorComponent],
    templateUrl: './dashboard.section.setting.component.html',
})
export class DashboardSectionSettingComponent implements OnInit {
    @Input({ required: true }) section!: DashboardSystemSection;

    readonly store = inject(DashboardSettingsService);

    readonly logoSetting = computed(() => this.store.getSettingByName('site.logo'));

    ngOnInit(): void {
        this.store.ensureLoaded();
    }

    getSectionLabel(): string {
        return this.store.getSectionLabel(this.section) || this.getFallbackLabel();
    }

    getSectionTitle(): string {
        const title = this.store.getSettingValue(this.getTitleKey());
        if (typeof title === 'string' && title.trim().length) {
            return title;
        }
        return this.getSectionLabel();
    }

    getSectionSubtitle(): string {
        const subtitle = this.store.getSettingValue(this.getSubtitleKey());
        if (typeof subtitle === 'string' && subtitle.trim().length) {
            return subtitle;
        }
        return 'Tweak how visitors explore Nimble Photos, control notifications, and manage uploads from one place.';
    }

    getSectionSettings(): DashboardSetting[] {
        return this.store.getSectionSettings(this.section);
    }

    getVisibleSettings(): DashboardSetting[] {
        return this.getSectionSettings().filter(setting => setting.key !== 'site.logo');
    }

    showLogoEditor(): boolean {
        return this.section === 'general' && !!this.logoSetting();
    }

    onLogoChanged(path: string): void {
        const setting = this.logoSetting();
        if (!setting) {
            return;
        }
        this.store.setLocalValue(setting.key, path);
        this.store.saveSetting(setting);
    }

    getLogoPath(): string {
        const value = this.logoSetting()?.value;
        return typeof value === 'string' ? value : '';
    }

    private getFallbackLabel(): string {
        switch (this.section) {
            case 'general':
                return 'General Settings';
            case 'photo-manage':
                return 'Photo Management';
            default:
                return 'Settings';
        }
    }

    private getTitleKey(): string {
        return `dashboard.${this.section}.title`;
    }

    private getSubtitleKey(): string {
        return `dashboard.${this.section}.subtitle`;
    }
}
