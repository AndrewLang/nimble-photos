import { Component, computed, effect, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { finalize } from 'rxjs';

import { AdminDashboardUser } from '../../models/security-admin.model';
import { AuthService } from '../../services/auth.service';
import { DashboardSettingsService } from '../../services/dashboard.setting.service';
import { SecurityAdminService } from '../../services/security-admin.service';
import { ActionSelectorComponent } from './action.selector.component';
import { SvgComponent } from '../svg/svg.component';

@Component({
    selector: 'mtx-security-setting',
    imports: [FormsModule, SvgComponent, ActionSelectorComponent],
    templateUrl: './security.setting.component.html',
})
export class SecuritySettingComponent {
    private readonly store = inject(DashboardSettingsService);
    private readonly securityAdminService = inject(SecurityAdminService);
    private readonly authService = inject(AuthService);

    readonly users = signal<AdminDashboardUser[]>([]);
    readonly activeTab = signal<'users' | 'roles'>('users');
    readonly userRoleSelections = signal<Record<string, string[]>>({});
    readonly loadingUsers = signal(false);
    readonly usersError = signal<string | null>(null);
    readonly userSaveState = signal<Record<string, { saving: boolean; error: string | null; success: boolean }>>({});

    readonly permissionsDraft = signal<Record<string, Record<string, boolean>>>({});
    readonly newRoleName = signal('');
    readonly permissionsError = signal<string | null>(null);
    readonly savingPermissions = signal(false);
    readonly permissionsSaved = signal(false);

    readonly actionDefinitions = [
        { key: 'dashboard.access', label: 'Dashboard access' },
        { key: 'settings.general.update', label: 'Update general settings' },
        { key: 'photos.upload', label: 'Upload photos' },
        { key: 'comments.create', label: 'Create comments' },
    ] as const;

    readonly rolePermissionsSetting = computed(() => this.store.getSettingByName('security.rolePermissions'));
    readonly roleNames = computed(() => {
        const fromPermissions = Object.keys(this.permissionsDraft());
        const fromUsers = this.users().flatMap(user => user.roles || []);
        const names = Array.from(new Set([...fromPermissions, ...fromUsers]));
        names.sort((a, b) => a.localeCompare(b));
        return names;
    });
    readonly hasPermissionsDraftChanges = computed(() => {
        const setting = this.rolePermissionsSetting();
        if (!setting) {
            return false;
        }
        const current = this.normalizePermissionMap(setting.value);
        const draft = this.buildPermissionsPayload(this.permissionsDraft());
        return JSON.stringify(current) !== JSON.stringify(draft);
    });

    constructor() {
        this.store.ensureLoaded();
        this.loadUsers();

        effect(() => {
            const setting = this.rolePermissionsSetting();
            if (!setting) {
                return;
            }
            this.permissionsDraft.set(this.normalizePermissionMap(setting.value));
        });
    }

    loadUsers(): void {
        this.loadingUsers.set(true);
        this.usersError.set(null);
        this.securityAdminService
            .getUsers()
            .pipe(finalize(() => this.loadingUsers.set(false)))
            .subscribe({
                next: users => {
                    const normalizedUsers = (Array.isArray(users) ? users : []).map(user => ({
                        ...user,
                        roles: Array.isArray(user.roles) ? user.roles : [],
                    }));
                    this.users.set(normalizedUsers);
                    this.userRoleSelections.set(
                        normalizedUsers.reduce<Record<string, string[]>>((acc, user) => {
                            acc[user.id] = [...user.roles];
                            return acc;
                        }, {}),
                    );
                },
                error: err => {
                    this.usersError.set(err.error?.message || 'Failed to load users.');
                },
            });
    }

    selectedRolesFor(userId: string): string[] {
        return this.userRoleSelections()[userId] ?? [];
    }

    isUserRoleSelected(userId: string, role: string): boolean {
        return this.selectedRolesFor(userId).includes(role);
    }

    toggleUserRole(userId: string, role: string, checked: boolean): void {
        if (!this.canToggleUserRole(userId, role)) {
            return;
        }

        const current = this.selectedRolesFor(userId);
        const next = checked ? Array.from(new Set([...current, role])) : current.filter(item => item !== role);
        this.userRoleSelections.update(all => ({ ...all, [userId]: next }));
        this.userSaveState.update(current => ({
            ...current,
            [userId]: { saving: false, error: null, success: false },
        }));
    }

    canToggleUserRole(userId: string, role: string): boolean {
        if (role !== 'admin') {
            return true;
        }
        if (!this.isCurrentUser(userId)) {
            return true;
        }
        return !this.isUserRoleSelected(userId, role);
    }

    saveUserRoles(user: AdminDashboardUser): void {
        const roles = this.selectedRolesFor(user.id)
            .map(role => role.trim().toLowerCase())
            .filter(role => /^[a-z0-9_-]+$/.test(role));

        if (!roles.length) {
            this.userSaveState.update(current => ({
                ...current,
                [user.id]: { saving: false, error: 'At least one role is required.', success: false },
            }));
            return;
        }

        this.userSaveState.update(current => ({
            ...current,
            [user.id]: { saving: true, error: null, success: false },
        }));

        this.securityAdminService.updateUserRoles(user.id, { roles }).subscribe({
            next: updated => {
                this.users.update(current => current.map(item => (item.id === updated.id ? updated : item)));
                this.userRoleSelections.update(current => ({ ...current, [updated.id]: [...updated.roles] }));
                this.userSaveState.update(current => ({
                    ...current,
                    [user.id]: { saving: false, error: null, success: true },
                }));

                window.setTimeout(() => {
                    this.userSaveState.update(current => ({
                        ...current,
                        [user.id]: {
                            saving: false,
                            error: current[user.id]?.error ?? null,
                            success: false,
                        },
                    }));
                }, 1800);
            },
            error: err => {
                this.userSaveState.update(current => ({
                    ...current,
                    [user.id]: {
                        saving: false,
                        error: err.error?.message || 'Failed to update roles.',
                        success: false,
                    },
                }));
            },
        });
    }

    roleActionValue(role: string, action: string): boolean {
        if (this.isAdminRole(role)) {
            return true;
        }
        return Boolean(this.permissionsDraft()[role]?.[action]);
    }

    addRole(): void {
        const role = this.newRoleName().trim().toLowerCase();
        if (!role.length) {
            return;
        }
        if (!/^[a-z0-9_-]+$/.test(role)) {
            this.permissionsError.set('Role must contain only letters, numbers, "-" or "_".');
            return;
        }

        this.permissionsDraft.update(current => {
            if (current[role]) {
                return current;
            }
            return {
                ...current,
                [role]: this.blankActions(),
            };
        });
        this.newRoleName.set('');
        this.permissionsError.set(null);
        this.permissionsSaved.set(false);
    }

    removeRole(role: string): void {
        this.permissionsDraft.update(current => {
            const next = { ...current };
            delete next[role];
            return next;
        });
        this.permissionsSaved.set(false);
    }

    savePermissions(): void {
        const setting = this.rolePermissionsSetting();
        if (!setting) {
            this.permissionsError.set('Role permissions setting was not found.');
            return;
        }

        const payload = this.buildPermissionsPayload(this.permissionsDraft());
        this.savingPermissions.set(true);
        this.permissionsError.set(null);
        this.permissionsSaved.set(false);

        this.store.setLocalValue(setting.key, JSON.stringify(payload, null, 2));
        this.store.saveSetting(setting);

        window.setTimeout(() => {
            this.savingPermissions.set(false);
            const fieldError = this.store.fieldErrors()[setting.key];
            if (fieldError) {
                this.permissionsError.set(fieldError);
                return;
            }
            this.permissionsSaved.set(true);
            window.setTimeout(() => this.permissionsSaved.set(false), 1800);
        }, 250);
    }

    usersInRole(role: string): number {
        return this.users().filter(user => user.roles?.includes(role)).length;
    }

    roleSelectedActionKeys(role: string): string[] {
        return this.actionDefinitions
            .filter(action => this.roleActionValue(role, action.key))
            .map(action => action.key);
    }

    onRoleActionsChange(role: string, selectedKeys: string[]): void {
        if (this.isAdminRole(role)) {
            return;
        }

        const selected = new Set(selectedKeys);
        this.permissionsDraft.update(current => ({
            ...current,
            [role]: this.actionDefinitions.reduce<Record<string, boolean>>((acc, action) => {
                acc[action.key] = selected.has(action.key);
                return acc;
            }, {}),
        }));
        this.permissionsError.set(null);
        this.permissionsSaved.set(false);
    }

    userSaveStatus(userId: string): { saving: boolean; error: string | null; success: boolean } {
        return this.userSaveState()[userId] || { saving: false, error: null, success: false };
    }

    trackUser = (_: number, user: AdminDashboardUser): string => user.id;

    selectTab(tab: 'users' | 'roles'): void {
        this.activeTab.set(tab);
    }

    isCurrentUser(userId: string): boolean {
        return this.authService.currentUser()?.id === userId;
    }

    private blankActions(): Record<string, boolean> {
        return this.actionDefinitions.reduce<Record<string, boolean>>((acc, action) => {
            acc[action.key] = false;
            return acc;
        }, {});
    }

    private normalizePermissionMap(value: unknown): Record<string, Record<string, boolean>> {
        if (!value || typeof value !== 'object' || Array.isArray(value)) {
            return {};
        }

        const source = value as Record<string, unknown>;
        const normalized: Record<string, Record<string, boolean>> = {};

        for (const role of Object.keys(source)) {
            const roleConfig = source[role];
            if (!roleConfig || typeof roleConfig !== 'object' || Array.isArray(roleConfig)) {
                normalized[role] = this.isAdminRole(role) ? this.fullActions() : this.blankActions();
                continue;
            }

            const roleConfigMap = roleConfig as Record<string, unknown>;
            const wildcard = Boolean(roleConfigMap['*']);
            normalized[role] = this.actionDefinitions.reduce<Record<string, boolean>>((acc, action) => {
                acc[action.key] = wildcard || Boolean(roleConfigMap[action.key]);
                return acc;
            }, {});
            if (this.isAdminRole(role)) {
                normalized[role] = this.fullActions();
            }
        }

        return normalized;
    }

    private buildPermissionsPayload(
        source: Record<string, Record<string, boolean>>,
    ): Record<string, Record<string, boolean>> {
        const payload: Record<string, Record<string, boolean>> = {};
        for (const role of Object.keys(source).sort((a, b) => a.localeCompare(b))) {
            payload[role] = this.isAdminRole(role)
                ? this.fullActions()
                : this.actionDefinitions.reduce<Record<string, boolean>>((acc, action) => {
                    acc[action.key] = Boolean(source[role]?.[action.key]);
                    return acc;
                }, {});
        }
        return payload;
    }

    isAdminRole(role: string): boolean {
        return role.trim().toLowerCase() === 'admin';
    }

    private fullActions(): Record<string, boolean> {
        return this.actionDefinitions.reduce<Record<string, boolean>>((acc, action) => {
            acc[action.key] = true;
            return acc;
        }, {});
    }
}
