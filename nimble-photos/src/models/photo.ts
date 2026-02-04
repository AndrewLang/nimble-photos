export interface PhotoMetadata {
  location?: string;
  camera?: string;
  aperture?: string;
  shutterSpeed?: string;
  iso?: number;
  focalLength?: string;
  aspectRatio?: string;
  dateCreated?: string;
  lat?: number;
  lng?: number;
}

export interface Photo {
  id: string;
  path: string;
  thumbnailPath?: string;
  name: string;
  format?: string;
  hash?: string;
  size?: number;
  createdAt?: Date;
  updatedAt?: Date;
  dateImported?: Date;
  dateTaken?: Date;
  thumbnailOptimized?: boolean;
  metadataExtracted?: boolean;
  isRaw?: boolean;
  width?: number;
  height?: number;
  thumbnailWidth?: number;
  thumbnailHeight?: number;
  metadata?: PhotoMetadata;
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

export type AlbumKind = 'manual' | 'smart';

export interface Album {
  id: string;
  parentId?: string;
  name: string;
  createDate?: Date;
  description?: string;
  category?: string;
  kind: AlbumKind;
  rulesJson?: string;
  thumbnailHash?: string;
  sortOrder: number;
  imageCount?: number;
  photos?: PagedPhotos;
}

export interface PhotoLoc extends Photo {
  lat: number;
  lon: number;
}
