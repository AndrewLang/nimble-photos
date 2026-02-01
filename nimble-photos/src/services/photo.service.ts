import { Injectable } from '@angular/core';
import { delay, Observable, of } from 'rxjs';

import { Album, GroupedPhotos, PagedPhotos, Photo, PhotoMetadata } from '../models/photo.model';

export interface PagedAlbums {
  page: number;
  pageSize: number;
  total: number;
  items: Album[];
}


type PhotoTemplate = Pick<Photo, 'title' | 'description' | 'tags'> & {
  url: string;
  metadata: Omit<PhotoMetadata, 'iso'>;
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
      dateCreated: '2023-11-15T08:00:00Z',
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
      dateCreated: '2023-12-01T14:30:00Z',
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
      dateCreated: '2023-10-20T16:45:00Z',
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
      dateCreated: '2024-01-05T09:15:00Z',
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
      dateCreated: '2023-09-12T19:20:00Z',
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
      dateCreated: '2024-02-14T07:30:00Z',
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
      dateCreated: '2023-08-30T10:00:00Z',
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
      dateCreated: '2023-11-02T18:10:00Z',
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
      dateCreated: '2023-07-25T12:00:00Z',
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
      dateCreated: '2023-09-05T15:40:00Z',
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
      dateCreated: '2024-01-20T17:00:00Z',
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
      dateCreated: '2023-12-31T23:59:00Z',
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
      dateCreated: '2023-10-15T11:20:00Z',
    },
  },
];

@Injectable({
  providedIn: 'root',
})
export class PhotoService {
  private readonly library = this.buildPhotoLibrary();
  private readonly albums = this.buildAlbums();

  getAlbums(page = 1, pageSize = 12): Observable<PagedAlbums> {
    const start = (page - 1) * pageSize;
    const sorted = [...this.albums].sort((a, b) => b.dateCreated.getTime() - a.dateCreated.getTime());
    const items = sorted.slice(start, start + pageSize);
    return of({
      page,
      pageSize,
      total: this.albums.length,
      items,
    }).pipe(delay(300));
  }

  getAlbumById(id: string): Observable<Album | null> {
    const album = this.albums.find((a) => a.id === id);
    return of(album || null).pipe(delay(200));
  }

  getAlbumPhotos(albumId: string, page = 1, pageSize = 20): Observable<PagedPhotos | null> {
    const album = this.albums.find((a) => a.id === albumId);
    if (!album) return of(null);

    // In this mock, all photos are already in album.photos.items. 
    // If we had more, we would slice them here.
    return of(album.photos).pipe(delay(200));
  }

  getPhotos(page = 1, pageSize = 56): Observable<PagedPhotos> {
    const start = (page - 1) * pageSize;
    const items = this.library.slice(start, start + pageSize);
    return of({
      page,
      pageSize,
      total: this.library.length,
      items,
    }).pipe(delay(220));
  }

  getGroupedPhotos(
    groupIndex: number,
    pageInGroup: number = 1,
    pageSize: number = 100
  ): Observable<GroupedPhotos | null> {

    // 1. Group entire library (in real app, this would be DB query)
    const allGroups = this.groupLibraryByYearMonth();

    if (groupIndex >= allGroups.length) {
      return of(null).pipe(delay(200));
    }

    const targetGroup = allGroups[groupIndex];
    if (pageInGroup > Math.ceil(targetGroup.items.length / pageSize)) {
      // Page out of range
      return of({
        title: targetGroup.title,
        photos: {
          page: pageInGroup, // Return empty page
          pageSize,
          total: targetGroup.items.length,
          items: []
        }
      }).pipe(delay(200));
    }

    const totalPhotos = targetGroup.items.length;

    // Pagination within group
    const start = (pageInGroup - 1) * pageSize;
    const items = targetGroup.items.slice(start, start + pageSize);

    return of({
      title: targetGroup.title,
      photos: {
        page: pageInGroup,
        pageSize,
        total: totalPhotos,
        items
      }
    }).pipe(delay(220));
  }

  private groupLibraryByYearMonth(): { title: string; items: Photo[] }[] {
    const groups: Record<string, Photo[]> = {};

    const sorted = [...this.library].sort((a, b) =>
      b.dateCreated.getTime() - a.dateCreated.getTime()
    );

    for (const p of sorted) {
      const key = p.dateCreated.toISOString().slice(0, 7); // "YYYY-MM"
      if (!groups[key]) groups[key] = [];
      groups[key].push(p);
    }

    return Object.keys(groups)
      .sort((a, b) => b.localeCompare(a))
      .map(key => ({ title: key, items: groups[key] }));
  }

  private buildAlbums(): Album[] {
    const albumCount = 15;
    const albums: Album[] = [];

    const stories = [
      'A journey through the misty mountains of the East, where the air is thin and the spirits are high.',
      'Summer days spent by the crystal clear lakes, reflecting the deepest blues of the sky.',
      'Exploring the neon-lit streets of a restless city that never sleeps, catching glimpses of hidden lives.',
      'A quiet retreat into the heart of the forest, rediscovering the rhythms of nature and the songs of birds.',
      'The vibrant colors of a coastal village at dusk, where the salt air meets the smell of fresh seafood.',
      'Architectural marvels that stand as witnesses to centuries past, their stones whispering secrets of history.',
      'Chasing the golden hour across rolling hills and open fields, where every ray of light tells a story.',
      'A winter tale in a snow-covered land, filled with frozen beauty and warm moments by the fire.',
      'Cultural celebrations and festivals that bring communities together in a burst of music and dance.',
      'The minimal elegance of desert landscapes, where silence becomes a powerful presence.',
    ];

    for (let i = 0; i < albumCount; i++) {
      const storyIndex = i % stories.length;
      const albumPhotos = this.library.slice(i * 10, (i + 1) * 10);
      const date = new Date(Date.now() - 3600 * 1000 * 24 * (i * 15 + 5));

      albums.push({
        id: `album-${i + 1}`,
        title: `Adventure ${i + 1}: ${albumPhotos[0]?.title || 'Untitled'}`,
        story: stories[storyIndex],
        coverPhotoUrl: albumPhotos[0]?.url || '',
        dateCreated: date,
        photos: {
          page: 1,
          pageSize: 10,
          total: albumPhotos.length,
          items: albumPhotos,
        },
      });
    }
    return albums;
  }

  private buildPhotoLibrary(): Photo[] {
    const targetSize = 3500;
    const photos: Photo[] = [];
    for (let index = 0; index < targetSize; index++) {
      const template = PHOTO_TEMPLATES[index % PHOTO_TEMPLATES.length];
      const variant = Math.floor(index / PHOTO_TEMPLATES.length) + 1;

      // Clustering logic:
      const clusterSize = 6;
      const daysPerCluster = 7; // Step back a week every cluster
      const clusterIndex = Math.floor(index / clusterSize);
      // Add a tiny random offset within the day so they sort deterministically but look natural
      const offsetWithinDay = (index % clusterSize) * 0.1;
      const totalDaysBack = (clusterIndex * daysPerCluster) + offsetWithinDay;
      const date = new Date(Date.now() - 3600 * 1000 * 24 * totalDaysBack);

      photos.push({
        id: `photo-${String(index + 1).padStart(4, '0')}`,
        url: `${template.url}?auto=format&fit=crop&w=900&q=80&fm=jpg&sig=${index + 1}`,
        dateCreated: date,
        title: `${template.title} ${variant}`,
        description: template.description,
        tags: template.tags,
        metadata: {
          ...template.metadata,
          iso: 100 + ((index * 13) % 700),
          dateCreated: date.toISOString(),
        },
      });
    }
    return photos;
  }
}
