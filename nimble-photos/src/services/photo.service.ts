import { Injectable } from '@angular/core';
import { delay, Observable, of } from 'rxjs';

import { Photo, PhotoMetadata, PhotoPage } from '../models/photo.model';

type PhotoTemplate = Pick<Photo, 'title' | 'description' | 'tags'> & {
  url: string;
  metadata: Omit<PhotoMetadata, 'capturedAt' | 'iso'>;
};

const PHOTO_TEMPLATES: PhotoTemplate[] = [
  {
    title: 'Lighthouse Calm',
    description: 'A lonely beacon brushes the gray line between sea and sky.',
    url: 'https://images.unsplash.com/photo-1507525428034-b723cf961d3e',
    tags: ['coast', 'dawn', 'ruck'],
    metadata: {
      location: 'Hokkaido, Japan',
      camera: 'Sony Alpha 7R IV',
      aperture: 'f/1.8',
      shutterSpeed: '1/320',
      focalLength: '35mm',
      aspectRatio: '3 / 4',
    },
  },
  {
    title: 'Fog Rail',
    description: 'Tracks disappear in a breath of mist and cold copper tones.',
    url: 'https://images.unsplash.com/photo-1469474968028-56623f02e42e',
    tags: ['rail', 'fog', 'minimal'],
    metadata: {
      location: 'Trans-Siberian, Russia',
      camera: 'Fujifilm GFX 50S',
      aperture: 'f/4.0',
      shutterSpeed: '1/250',
      focalLength: '45mm',
      aspectRatio: '2 / 3',
    },
  },
  {
    title: 'Golden Paths',
    description: 'Afternoon sun turns the meadow into a quilt of gold.',
    url: 'https://images.unsplash.com/photo-1500530855697-b586d89ba3ee',
    tags: ['meadow', 'golden hour', 'nature'],
    metadata: {
      location: 'Tuscany, Italy',
      camera: 'Canon EOS R6',
      aperture: 'f/2.8',
      shutterSpeed: '1/160',
      focalLength: '55mm',
      aspectRatio: '4 / 5',
    },
  },
  {
    title: 'Temple Stillness',
    description: 'Gilded pillars shelter the silence of an ancient courtyard.',
    url: 'https://images.unsplash.com/photo-1470770841072-f978cf4d019e',
    tags: ['architecture', 'zen', 'culture'],
    metadata: {
      location: 'Kyoto, Japan',
      camera: 'Nikon Z6',
      aperture: 'f/5.6',
      shutterSpeed: '1/60',
      focalLength: '24mm',
      aspectRatio: '3 / 4',
    },
  },
  {
    title: 'City Steps',
    description: 'Warm dusk light spills over the concrete geometry.',
    url: 'https://images.unsplash.com/photo-1494526585095-c41746248156',
    tags: ['city', 'evening', 'minimal'],
    metadata: {
      location: 'Lisbon, Portugal',
      camera: 'Leica M10',
      aperture: 'f/2.2',
      shutterSpeed: '1/200',
      focalLength: '35mm',
      aspectRatio: '2 / 3',
    },
  },
  {
    title: 'Monk’s Pause',
    description: 'Stone guardians stare across the courtyard of a quiet monastery.',
    url: 'https://images.unsplash.com/photo-1500534314209-a25ddb2bd429',
    tags: ['statue', 'monk', 'stillness'],
    metadata: {
      location: 'Bhutan',
      camera: 'Canon EOS R5',
      aperture: 'f/2.0',
      shutterSpeed: '1/400',
      focalLength: '85mm',
      aspectRatio: '9 / 16',
    },
  },
  {
    title: 'Prairie Gathering',
    description: 'Families gather along the winding trail as dusk paints the grasses.',
    url: 'https://images.unsplash.com/photo-1524504388940-b1c1722653e1',
    tags: ['community', 'outdoors', 'joy'],
    metadata: {
      location: 'Nebraska, USA',
      camera: 'Pentax 645Z',
      aperture: 'f/6.3',
      shutterSpeed: '1/125',
      focalLength: '70mm',
      aspectRatio: '4 / 3',
    },
  },

  {
    title: 'Copper Lantern',
    description: 'Lantern light whispers from the treetops, written characters glowing.',
    url: 'https://images.unsplash.com/photo-1681215919198-83896a9ead7d',
    tags: ['lantern', 'calligraphy', 'autumn'],
    metadata: {
      location: 'Hangzhou, China',
      camera: 'Sony A7 IV',
      aperture: 'f/4.5',
      shutterSpeed: '1/60',
      focalLength: '50mm',
      aspectRatio: '3 / 4',
    },
  },
  {
    title: 'Harbor Study',
    description: 'Cobalt isles punctuate the horizon behind a reflective boardwalk.',
    url: 'https://images.unsplash.com/photo-1551259510-6c2adc679053',
    tags: ['harbor', 'water', 'architecture'],
    metadata: {
      location: 'Lisbon, Portugal',
      camera: 'Nikon D850',
      aperture: 'f/8.0',
      shutterSpeed: '1/320',
      focalLength: '40mm',
      aspectRatio: '4 / 3',
    },
  },
  {
    title: 'Cloud White',
    description: 'A tally of cloudbanks drifts above a lonely berm.',
    url: 'https://images.unsplash.com/photo-1627307285965-ad88240f7bca',
    tags: ['cloud', 'sky', 'serene'],
    metadata: {
      location: 'Mongolia',
      camera: 'Fuji X-Pro3',
      aperture: 'f/5.6',
      shutterSpeed: '1/500',
      focalLength: '35mm',
      aspectRatio: '5 / 4',
    },
  },
  {
    title: 'Ferris Memory',
    description: 'A wheel of lights hovers against a pastel afternoon sky.',
    url: 'https://images.unsplash.com/photo-1494790108377-be9c29b29330',
    tags: ['fair', 'pastel', 'calm'],
    metadata: {
      location: 'Paris, France',
      camera: 'Olympus OM-D E-M1 Mark III',
      aperture: 'f/4.0',
      shutterSpeed: '1/250',
      focalLength: '45mm',
      aspectRatio: '9 / 16',
    },
  },
  {
    title: 'Crowd Chant',
    description: 'Young voices raise banners beneath cool laser lights.',
    url: 'https://images.unsplash.com/photo-1544894079-e81a9eb1da8b',
    tags: ['crowd', 'culture', 'festival'],
    metadata: {
      location: 'Bangkok, Thailand',
      camera: 'Canon EOS 5D',
      aperture: 'f/2.8',
      shutterSpeed: '1/160',
      focalLength: '30mm',
      aspectRatio: '3 / 4',
    },
  },
  {
    title: 'Sunlit Way',
    description: 'Tree-lined avenues collect every golden leaf that falls.',
    url: 'https://images.unsplash.com/photo-1546464677-c25cd52c470b',
    tags: ['autumn', 'trees', 'silence'],
    metadata: {
      location: 'Seoul, South Korea',
      camera: 'Sony A7S III',
      aperture: 'f/4.0',
      shutterSpeed: '1/200',
      focalLength: '35mm',
      aspectRatio: '3 / 5',
    },
  },
];

@Injectable({
  providedIn: 'root',
})
export class PhotoService {
  private readonly library = this.buildPhotoLibrary();

  getPhotos(page = 1, pageSize = 56): Observable<PhotoPage> {
    const start = (page - 1) * pageSize;
    const items = this.library.slice(start, start + pageSize);
    return of({
      page,
      pageSize,
      total: this.library.length,
      items,
    }).pipe(delay(220));
  }

  private buildPhotoLibrary(): Photo[] {
    const targetSize = 1520;
    const photos: Photo[] = [];
    for (let index = 0; index < targetSize; index++) {
      const template = PHOTO_TEMPLATES[index % PHOTO_TEMPLATES.length];
      const variant = Math.floor(index / PHOTO_TEMPLATES.length) + 1;
      photos.push({
        id: `photo-${String(index + 1).padStart(4, '0')}`,
        url: `${template.url}?auto=format&fit=crop&w=900&q=80&fm=jpg&sig=${index + 1}`,
        title: `${template.title} ${variant}`,
        description: template.description,
        tags: template.tags,
        metadata: {
          ...template.metadata,
          iso: 100 + ((index * 13) % 700),
          capturedAt: new Date(Date.now() - 3600 * 1000 * index).toISOString(),
        },
      });
    }
    return photos;
  }
}
