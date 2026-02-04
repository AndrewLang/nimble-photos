export interface PhotoMetadata {
  id?: string;
  imageId?: string;
  hash?: string;
  make?: string;
  model?: string;
  lensMake?: string;
  lensModel?: string;
  lensSerialNumber?: string;
  lensSpecification?: string;
  bodySerialNumber?: string;
  exposureTime?: string;
  exposureProgram?: string;
  exposureMode?: string;
  exposureBiasValue?: number;
  fNumber?: number;
  apertureValue?: number;
  maxApertureValue?: number;
  brightnessValue?: number;
  shutterSpeedValue?: number;
  iso?: number;
  sensitivityType?: string;
  recommendedExposureIndex?: number;
  meteringMode?: string;
  lightSource?: string;
  flash?: string;
  exposureIndex?: number;
  gainControl?: string;
  subjectDistance?: number;
  focalLength?: number;
  focalLengthIn35mmFilm?: number;
  colorSpace?: string;
  bitsPerSample?: string;
  imageWidth?: number;
  imageLength?: number;
  pixelXDimension?: number;
  pixelYDimension?: number;
  xResolution?: number;
  yResolution?: number;
  resolutionUnit?: string;
  compression?: string;
  orientation?: number;
  digitalZoomRatio?: number;
  whiteBalance?: string;
  contrast?: string;
  saturation?: string;
  sharpness?: string;
  customRendered?: string;
  sceneCaptureType?: string;
  sceneType?: string;
  subjectDistanceRange?: string;
  rating?: number;
  label?: string;
  flagged?: number;
  whitePoint?: string;
  primaryChromaticities?: string;
  transferFunction?: string;
  gamma?: number;
  datetime?: string;
  datetimeOriginal?: string;
  datetimeDigitized?: string;
  offsetTime?: string;
  offsetTimeOriginal?: string;
  offsetTimeDigitized?: string;
  subsecTime?: string;
  subsecTimeOriginal?: string;
  subsecTimeDigitized?: string;
  gpsLatitude?: number;
  gpsLongitude?: number;
  gpsAltitude?: number;
  gpsAltitudeRef?: string;
  gpsLatitudeRef?: string;
  gpsLongitudeRef?: string;
  gpsSpeed?: number;
  gpsSpeedRef?: string;
  gpsImgDirection?: number;
  gpsImgDirectionRef?: string;
  gpsDateStamp?: string;
  gpsTimeStamp?: string;
  gpsProcessingMethod?: string;
  gpsAreaInformation?: string;
  software?: string;
  artist?: string;
  copyright?: string;
  userComment?: string;
  makerNote?: string;
  fileSource?: string;
  sensingMethod?: string;
  cfaPattern?: string;
  photographicSensitivity?: number;
  interopIndex?: string;
  interopVersion?: string;
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
