import { Component, Input, OnInit, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { DashboardSetting, DashboardSystemSection } from '../../models/dashboard.settings.model';
import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-dashboard-section-setting',
    imports: [FormsModule, SvgComponent],
    templateUrl: './dashboard.section.setting.component.html',
})
export class DashboardSectionSettingComponent implements OnInit {
    @Input({ required: true }) section!: DashboardSystemSection;

    readonly store = inject(DashboardSettingsService);

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
