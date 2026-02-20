export interface Command {
    id: string;
    name: string;
    description?: string;
    icon?: string;
    isHidden?: boolean;
    action?: (context?: any) => void;
}