import { Photo } from './photo';

export type DateFieldKey = 'createdAt' | 'updatedAt' | 'dateImported' | 'dateTaken';

export type PhotoResponse = Omit<Photo, DateFieldKey> & {
  createdAt?: string | null;
  updatedAt?: string | null;
  dateImported?: string | null;
  dateTaken?: string | null;
};

export type PhotoLocResponse = PhotoResponse & {
  lat: number;
  lon: number;
};
