export interface AdminDashboardUser {
    id: string;
    email: string;
    displayName: string;
    createdAt: string;
    emailVerified: boolean;
    roles: string[];
}

export interface UpdateUserRolesRequest {
    roles: string[];
}
