export interface DashboardClient {
    id: string;
    userId: string;
    name: string;
    isActive: boolean;
    isApproved: boolean;
    lastSeenAt?: string | null;
    createdAt: string;
    updatedAt: string;
}
