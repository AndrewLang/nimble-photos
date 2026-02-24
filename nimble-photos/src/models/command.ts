export interface Command {
    id: string;
    name: string;
    description?: string;
    icon?: string;
    isHidden?: boolean;
    action?: (context?: any) => void;
}

export interface Nav {
    id: string;
    label: string;
    route?: string;
    icon?: string;
    exact?: boolean;
    isHidden?: boolean;
    action?: (context?: any) => void;
}
