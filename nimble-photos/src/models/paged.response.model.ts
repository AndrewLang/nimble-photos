export interface PagedModel<T> {
    items: T[];
    total: number;
    page: number;
    pageSize: number;
}
