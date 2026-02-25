export const DashboardSystemSections = {
    General: 'general',
    Experience: 'experience',
    Notifications: 'notifications',
    Security: 'security',
    PhotoManage: 'photo-manage',
    Storage: 'storage',
    Client: 'client'
} as const;

export type DashboardSystemSection =
    typeof DashboardSystemSections[keyof typeof DashboardSystemSections];


export const DashboardSettingValueTypes = {
    String: 'string',
    Boolean: 'boolean',
    Number: 'number',
    Json: 'json'
} as const;

export type DashboardSettingValueType =
    typeof DashboardSettingValueTypes[keyof typeof DashboardSettingValueTypes];

export interface DashboardSettingOption {
    label: string;
    value: string | number | boolean | null;
}

export interface DashboardSetting {
    key: string;
    label: string;
    description: string;
    section: DashboardSystemSection;
    sectionLabel: string;
    group: string;
    valueType: DashboardSettingValueType;
    value: unknown;
    defaultValue: unknown;
    updatedAt: string;
    options?: DashboardSettingOption[];
}
