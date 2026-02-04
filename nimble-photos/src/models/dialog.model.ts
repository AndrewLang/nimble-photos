import { Type } from '@angular/core';

export interface DialogAction {
    label: string;
    value: any;
    style?: 'primary' | 'secondary' | 'danger' | 'ghost';
    closeDialog?: boolean;
}

export interface DialogConfig {
    title?: string;
    width?: string;
    maxWidth?: string;
    actions?: DialogAction[];
    showCloseButton?: boolean;
    closeOnBackdropClick?: boolean;
    data?: any;
}

export interface DialogRef<T = any> {
    close(result?: T): void;
    afterClosed(): Promise<T | undefined>;
    componentInstance?: any;
}

export interface ConfirmDialogData {
    title: string;
    message: string;
    confirmText?: string;
    cancelText?: string;
    type?: 'danger' | 'info' | 'warning';
}
