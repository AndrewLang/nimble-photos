import { Component, input, OnInit } from '@angular/core';

@Component({
    selector: 'mtx-spinner',
    templateUrl: 'spinner.component.html',
})
export class SpinnerComponent implements OnInit {
    size = input<number>(24);
    label = input<string>('Loading');
    showLabel = input<boolean>(false);

    constructor() { }

    ngOnInit() { }
}
