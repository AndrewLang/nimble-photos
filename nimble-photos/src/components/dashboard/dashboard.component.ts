
import { Component, computed, inject, OnInit } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

import { DashboardSystemSection } from '../../models/dashboard.settings.model';
import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-dashboard',
    imports: [RouterLink, RouterLinkActive, RouterOutlet, SvgComponent],
    templateUrl: './dashboard.component.html',
    host: {
        class: 'flex-1 flex flex-col min-h-0 overflow-hidden',
    },
})
export class DashboardComponent implements OnInit {
    readonly store = inject(DashboardSettingsService);
    readonly navSections = computed(() =>
        this.sections.map(section => ({
            section,
            label: this.store.getSectionLabel(section) || this.getFallbackLabel(section),
            count: this.store.getSectionCount(section),
        })),
    );

    private readonly sections: DashboardSystemSection[] = ['general', 'security', 'photo-manage', 'storage'];

    getSectionIcon(section: DashboardSystemSection): string {
        switch (section) {
            case 'general':
                return 'sectionGeneral';
            case 'photo-manage':
                return 'sectionPhotoManage';
            case 'storage':
                return 'sectionStorage';
            case 'security':
                return 'sectionDefault';
            default:
                return 'sectionDefault';
        }
    }

    ngOnInit(): void {
        this.store.ensureLoaded();
    }

    getFallbackLabel(section: DashboardSystemSection): string {
        switch (section) {
            case 'general':
                return 'General Settings';
            case 'photo-manage':
                return 'Photo Management';
            case 'storage':
                return 'Storage Manage';
            case 'security':
                return 'Role & Security';
            default:
                return 'Settings';
        }
    }
}
