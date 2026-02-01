export interface PhotoMetadata {
  location: string;
  camera: string;
  aperture: string;
  shutterSpeed: string;
  iso: number;
  focalLength: string;
  aspectRatio: string;
  dateCreated: string;
}

export interface Photo {
  id: string;
  url: string;
  dateCreated: Date;
  title: string;
  description: string;
  tags: string[];
  metadata: PhotoMetadata;
}

export interface PagedPhotos {
  page: number;
  pageSize: number;
  total: number;
  items: Photo[];
}

export interface GroupedPhotos {
  title: string;
  photos: PagedPhotos;
}
