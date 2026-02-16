import { Component, Input, OnInit, signal } from '@angular/core';

import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';
import { Photo } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';
import { SvgComponent } from '../svg/svg.component';

@Component({
  selector: 'mtx-album-editor',
  standalone: true,
  imports: [FormsModule, ReactiveFormsModule, SvgComponent],
  templateUrl: './album.editor.component.html'
})
export class AlbumEditorComponent implements OnInit {
  @Input() photos: Photo[] = [];

  readonly selectedPhotos = signal<Photo[]>([]);
  albumForm!: FormGroup;

  constructor(
    private readonly fb: FormBuilder,
    private readonly photoService: PhotoService
  ) {
    this.albumForm = this.fb.group({
      name: ['', [Validators.required, Validators.minLength(1)]],
      description: ['']
    });
  }

  ngOnInit() {
    this.selectedPhotos.set([...this.photos]);
  }

  removePhoto(photo: Photo) {
    this.selectedPhotos.update(photos => photos.filter(p => p.id !== photo.id));
  }

  getThumbnail(photo: Photo) {
    return this.photoService.getThumbnailPath(photo);
  }

  getFormValue() {
    return {
      ...this.albumForm.value,
      photoIds: this.selectedPhotos().map(p => p.id)
    };
  }

  isValid() {
    return this.albumForm.valid && this.selectedPhotos().length > 0;
  }
}
