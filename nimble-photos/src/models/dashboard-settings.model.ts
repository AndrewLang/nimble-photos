export type DashboardSystemSection = 'general' | 'experience' | 'notifications';
export type DashboardSettingValueType = 'string' | 'boolean' | 'number' | 'json';

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
