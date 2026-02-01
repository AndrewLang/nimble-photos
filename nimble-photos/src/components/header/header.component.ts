import { Component } from '@angular/core';
import { RouterModule } from '@angular/router';

@Component({
  selector: 'mtx-header',
  imports: [RouterModule],
  templateUrl: './header.component.html',
  styles: [`
    :host {
      display: block;
      width: 100%;
    }
  `]
})
export class HeaderComponent { }
