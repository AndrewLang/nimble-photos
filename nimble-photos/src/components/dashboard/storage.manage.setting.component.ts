import { Component } from '@angular/core';
import { StorageManageComponent } from '../storage/storage.manage.component';

@Component({
    selector: 'mtx-dashboard-storage-manage',
    imports: [StorageManageComponent],
    templateUrl: './storage.manage.setting.component.html',
    host: {
        class: 'flex flex-1 flex-col min-h-0',
    },
})
export class StorageManageSettingComponent { }
