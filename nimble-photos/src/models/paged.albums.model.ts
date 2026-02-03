import { Album } from './photo';

export interface PagedAlbumsModel {
    page: number;
    pageSize: number;
    total: number;
    items: Album[];
}
