import { Album } from './photo.model';

export interface PagedAlbumsModel {
    page: number;
    pageSize: number;
    total: number;
    items: Album[];
}
