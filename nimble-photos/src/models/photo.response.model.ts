import { Photo } from './photo';

export type DateFieldKey = 'createdAt' | 'updatedAt' | 'dateImported' | 'dateTaken' | 'sortDate' | 'dayDate';

export type PhotoResponse = Omit<Photo, DateFieldKey> & {
  createdAt?: string | null;
  updatedAt?: string | null;
  dateImported?: string | null;
  dateTaken?: string | null;
  sortDate?: string | null;
  dayDate?: string | null;
};

export type PhotoLocResponse = PhotoResponse & {
  lat: number;
  lon: number;
};
