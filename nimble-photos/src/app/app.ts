import { Component } from '@angular/core';
import { GalleryComponent } from '../components/gallery/gallery.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [GalleryComponent],
  templateUrl: './app.html',
  styleUrls: ['./app.css']
})
export class App {}
