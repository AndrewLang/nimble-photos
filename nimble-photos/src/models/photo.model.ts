export interface PhotoMetadata {
  location: string;
  camera: string;
  aperture: string;
  shutterSpeed: string;
  iso: number;
  focalLength: string;
  aspectRatio: string;
  dateCreated: string;
  lat?: number;
  lng?: number;
}

export interface Photo {
  id: string;
  path: string;
  thumbnailPath: string;
  url: string;
  name: string;
  title: string;
  description: string;
  story: string;
  tags: string[];
  format: string;
  hash: string;
  size: number;
  createdAt: Date;
  updatedAt: Date;
  dateImported: Date;
  dateTaken: Date;
  thumbnailOptimized: boolean;
  metadataExtracted: boolean;
  isRaw: boolean;
  width: number;
  height: number;
  thumbnailWidth: number;
  thumbnailHeight: number;
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

export interface Album {
  id: string;
  title: string;
  story: string;
  coverPhotoUrl: string;
  dateCreated: Date;
  photos: PagedPhotos;
}
