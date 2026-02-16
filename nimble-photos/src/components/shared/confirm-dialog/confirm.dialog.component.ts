import { Component } from '@angular/core';
@Component({
    selector: 'mtx-confirm-dialog',
    templateUrl: './confirm.dialog.component.html',
})
export class ConfirmDialogComponent {
    message: string = '';
    type: 'danger' | 'info' | 'warning' = 'info';
}

