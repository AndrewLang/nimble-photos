
import { Component, computed, inject, OnInit } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

import { DashboardSystemSection, DashboardSystemSections } from '../../models/dashboard.settings.model';
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

    private readonly sections: DashboardSystemSection[] = [
        DashboardSystemSections.General,
        DashboardSystemSections.Security,
        DashboardSystemSections.PhotoManage,
        DashboardSystemSections.Storage,
        DashboardSystemSections.Client
    ];

    getSectionIcon(section: DashboardSystemSection): string {
        switch (section) {
            case DashboardSystemSections.General:
                return 'sectionGeneral';
            case DashboardSystemSections.PhotoManage:
                return 'sectionPhotoManage';
            case DashboardSystemSections.Storage:
                return 'sectionStorage';
            case DashboardSystemSections.Security:
                return 'sectionSecurity';
            case DashboardSystemSections.Client:
                return 'sectionClient';
            default:
                return 'sectionDefault';
        }
    }

    ngOnInit(): void {
        this.store.ensureLoaded();
    }

    getFallbackLabel(section: DashboardSystemSection): string {
        switch (section) {
            case DashboardSystemSections.General:
                return 'General Settings';
            case DashboardSystemSections.PhotoManage:
                return 'Photo Management';
            case DashboardSystemSections.Storage:
                return 'Storage Manage';
            case DashboardSystemSections.Security:
                return 'Role & Security';
            case DashboardSystemSections.Client:
                return 'Client Manage';
            default:
                return 'Settings';
        }
    }
}
