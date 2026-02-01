export interface PhotoMetadata {
  location: string;
  camera: string;
  aperture: string;
  shutterSpeed: string;
  iso: number;
  focalLength: string;
  aspectRatio: string;
  capturedAt: string;
}

export interface Photo {
  id: string;
  url: string;
  title: string;
  description: string;
  tags: string[];
  metadata: PhotoMetadata;
}

export interface PhotoPage {
  page: number;
  pageSize: number;
  total: number;
  items: Photo[];
}
