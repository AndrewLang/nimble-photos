import { Component, inject, signal } from '@angular/core';
import { finalize } from 'rxjs';

import { DashboardClient } from '../../models/client.model';
import { ClientAdminService } from '../../services/client-admin.service';

@Component({
    selector: 'mtx-client-manage-setting',
    templateUrl: './client.manage.setting.component.html',
})
export class ClientManageSettingComponent {
    private readonly clientService = inject(ClientAdminService);

    readonly clients = signal<DashboardClient[]>([]);
    readonly loading = signal(false);
    readonly error = signal<string | null>(null);
    readonly pendingAction = signal<Record<string, boolean>>({});

    constructor() {
        this.loadClients();
    }

    loadClients(): void {
        this.loading.set(true);
        this.error.set(null);
        this.clientService
            .listClients()
            .pipe(finalize(() => this.loading.set(false)))
            .subscribe({
                next: clients => this.clients.set(clients),
                error: err => this.error.set(err.error?.message || 'Failed to load clients.'),
            });
    }

    approve(clientId: string): void {
        this.setPending(clientId, true);
        this.clientService
            .approveClient(clientId)
            .pipe(finalize(() => this.setPending(clientId, false)))
            .subscribe({
                next: updated => {
                    this.clients.update(current => current.map(client => client.id === updated.id ? updated : client));
                },
                error: err => this.error.set(err.error?.message || 'Failed to approve client.'),
            });
    }

    revoke(clientId: string): void {
        this.setPending(clientId, true);
        this.clientService
            .revokeClient(clientId)
            .pipe(finalize(() => this.setPending(clientId, false)))
            .subscribe({
                next: updated => {
                    this.clients.update(current => current.map(client => client.id === updated.id ? updated : client));
                },
                error: err => this.error.set(err.error?.message || 'Failed to revoke client.'),
            });
    }

    isPending(clientId: string): boolean {
        return Boolean(this.pendingAction()[clientId]);
    }

    private setPending(clientId: string, value: boolean): void {
        this.pendingAction.update(current => ({ ...current, [clientId]: value }));
    }
}
