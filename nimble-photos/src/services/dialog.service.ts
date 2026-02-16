import { Injectable, Type, inject } from '@angular/core';
import { DialogComponent } from '../components/dialog/dialog.component';
import { DialogConfig, DialogRef } from '../models/dialog.model';
import { Dialog as CdkDialog } from '@angular/cdk/dialog';

@Injectable({
    providedIn: 'root'
})
export class DialogService {
    private readonly cdkDialog = inject(CdkDialog);

    open<T = any>(component: Type<any>, config: DialogConfig = {}): DialogRef<T> {
        const dialogRef = this.cdkDialog.open<T>(DialogComponent, {
            data: { component, config },
            backdropClass: ['backdrop-blur-sm', 'bg-black/60', 'transition-all', 'duration-300'],
            panelClass: ['outline-none', 'bg-transparent', 'border-none', 'shadow-none', 'p-4', 'flex', 'items-center', 'justify-center'],
            disableClose: config.closeOnBackdropClick !== false,
            width: config.width || 'auto',
            maxWidth: config.maxWidth || '95vw'
        } as any);

        return {
            close: (result?: T) => dialogRef.close(result),
            afterClosed: () => {
                return new Promise((resolve) => {
                    dialogRef.closed.subscribe(res => resolve(res as T));
                });
            },
            get componentInstance() {
                const shell = dialogRef.componentInstance as DialogComponent;
                return shell ? shell.componentInstance() : null;
            }
        };
    }

}
