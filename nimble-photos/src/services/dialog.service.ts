import { ComponentRef, Injectable, Type, ViewContainerRef, inject, ApplicationRef, createComponent, EnvironmentInjector } from '@angular/core';
import { DialogComponent } from '../components/dialog/dialog.component';
import { DialogConfig, DialogRef } from '../models/dialog.model';

@Injectable({
    providedIn: 'root'
})
export class DialogService {
    private readonly appRef = inject(ApplicationRef);
    private readonly injector = inject(EnvironmentInjector);

    open<T = any>(component: Type<any>, config: DialogConfig = {}): DialogRef<T> {
        // Create the dialog component
        const dialogComponentRef = createComponent(DialogComponent, {
            environmentInjector: this.injector
        });

        // Set up the dialog properties
        dialogComponentRef.instance.contentComponent = component;
        dialogComponentRef.instance.config.set(config);

        // Promise for afterClosed
        let afterClosedResolve: (value?: T) => void;
        const afterClosedPromise = new Promise<T | undefined>((resolve) => {
            afterClosedResolve = resolve;
        });

        // Handle close logic
        const close = (result?: T) => {
            this.appRef.detachView(dialogComponentRef.hostView);
            dialogComponentRef.destroy();
            afterClosedResolve(result);
        };

        dialogComponentRef.instance.onClose = close;

        // Attach to app
        this.appRef.attachView(dialogComponentRef.hostView);
        const domElem = (dialogComponentRef.hostView as any).rootNodes[0] as HTMLElement;
        document.body.appendChild(domElem);

        return {
            close,
            afterClosed: () => afterClosedPromise,
            get componentInstance() {
                return dialogComponentRef.instance.componentInstance();
            }
        };
    }
}
