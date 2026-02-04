import { Component } from '@angular/core';
import { RouterModule } from '@angular/router';
import { HeaderComponent } from '../components/header/header.component';

@Component({
  selector: 'mtx-root',
  imports: [RouterModule, HeaderComponent],
  templateUrl: './app.html'
})
export class App { }
