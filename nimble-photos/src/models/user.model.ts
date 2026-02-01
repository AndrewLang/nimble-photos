export interface User {
    id: string;
    email: string;
    displayName: string;
    createdAt: string;
    emailVerified: boolean;
    resetToken?: string;
    resetTokenExpiresAt?: string;
    verificationToken?: string;
}

export interface UserProfile {
    id: string;
    email: string;
    displayName: string;
    avatarUrl: string | null;
    theme: string;
    language: string;
    timezone: string;
}
