import { Component, OnInit, signal, computed, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { StorageService } from '../../services/storage.service';
import { StorageLocation } from '../../models/storage.model';
import { SvgComponent } from '../svg/svg.component';
import { Formatter } from '../../models/formatters';

@Component({
    selector: 'mtx-storage-selector',
    imports: [FormsModule, SvgComponent],
    templateUrl: './storage.selector.component.html',
})
export class StorageSelectorComponent implements OnInit {
    private readonly storageService = inject(StorageService);

    readonly locations = signal<StorageLocation[]>([]);
    readonly searchQuery = signal('');
    readonly selectedLocation = signal<StorageLocation | null>(null);
    readonly loading = signal(false);

    readonly filteredLocations = computed(() => {
        const query = this.searchQuery().trim().toLowerCase();
        const items = this.locations();
        if (!query) {
            return this.sortedLocations(items);
        }
        return this.sortedLocations(
            items.filter(location => {
                return (
                    location.label.toLowerCase().includes(query) ||
                    location.path.toLowerCase().includes(query)
                );
            }),
        );
    });
    readonly formatBytes = Formatter.formatBytes;
    readonly formatAvailablePercent = Formatter.formatAvailablePercent;

    ngOnInit(): void {
        this.fetchLocations();
    }

    private fetchLocations(): void {
        this.loading.set(true);
        this.storageService.getLocations().subscribe({
            next: (locations) => {
                this.locations.set(locations);
                const defaultLocation = locations.find(location => location.isDefault) ?? null;
                this.selectedLocation.set(defaultLocation);
                this.loading.set(false);
            },
            error: () => {
                this.locations.set([]);
                this.loading.set(false);
            },
        });
    }

    selectLocation(location: StorageLocation): void {
        this.selectedLocation.set(location);
    }

    isValid(): boolean {
        return this.selectedLocation() !== null;
    }

    getFormValue(): StorageLocation | null {
        return this.selectedLocation();
    }

    private sortedLocations(locations: StorageLocation[]): StorageLocation[] {
        return [...locations].sort((a, b) => {
            if (a.isDefault && !b.isDefault) {
                return -1;
            }
            if (!a.isDefault && b.isDefault) {
                return 1;
            }
            return a.label.localeCompare(b.label);
        });
    }
}
