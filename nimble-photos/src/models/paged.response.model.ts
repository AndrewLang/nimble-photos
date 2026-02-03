export interface PagedResponseModel<T> {
    items: T[];
    total: number;
    page: number;
    pageSize: number;
}
