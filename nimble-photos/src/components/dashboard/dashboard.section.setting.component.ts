import { Component, Input, OnInit, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { DashboardSetting, DashboardSystemSection } from '../../models/dashboard.settings.model';
import { DashboardSettingsStore } from '../../services/dashboard.setting.store';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-dashboard-section-setting',
    imports: [FormsModule, SvgComponent],
    templateUrl: './dashboard.section.setting.component.html',
})
export class DashboardSectionSettingComponent implements OnInit {
    @Input({ required: true }) section!: DashboardSystemSection;

    readonly store = inject(DashboardSettingsStore);

    ngOnInit(): void {
        this.store.ensureLoaded();
    }

    getSectionLabel(): string {
        return this.store.getSectionLabel(this.section) || this.getFallbackLabel();
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
}


