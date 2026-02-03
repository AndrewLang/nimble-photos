import { Component } from '@angular/core';
import { RouterModule } from '@angular/router';
import { HeaderComponent } from '../components/header/header.component';
import { GroupedGallery } from '../components/grouped.gallery/grouped.gallery';

@Component({
  selector: 'mtx-root',
  imports: [RouterModule, HeaderComponent, GroupedGallery],
  templateUrl: './app.html'
})
export class App { }
