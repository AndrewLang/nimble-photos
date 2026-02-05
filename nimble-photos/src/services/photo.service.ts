import { HttpClient } from '@angular/common/http';
import { Injectable, signal } from '@angular/core';
import { catchError, map, Observable, of, switchMap, tap } from 'rxjs';

import { AlbumModel } from '../models/album.model';
import { PagedAlbumsModel } from '../models/paged.albums.model';
import { PagedModel } from '../models/paged.response.model';
import { Album, AlbumComment, GroupedPhotos, PagedPhotos, Photo, PhotoComment, PhotoLoc, PhotoMetadata } from '../models/photo';
import { PhotoLocResponse, PhotoResponse } from '../models/photo.response.model';
import { API_BASE_URL } from './api.config';

@Injectable({
  providedIn: 'root',
})
export class PhotoService {
  readonly apiBase = API_BASE_URL;
  private timelinePhotoIds: string[] | null = null;
  public timelineCache: GroupedPhotos[] | null = null;

  lastGalleryScrollIndex = 0;
  readonly isScrolled = signal(false);

  constructor(private readonly http: HttpClient) { }

  getPhotos(page = 1, pageSize = 56): Observable<PagedPhotos> {
    return this.http
      .get<PagedModel<PhotoResponse>>(`${this.apiBase}/photos/${page}/${pageSize}`)
      .pipe(map((response) => this.mapPhotoPage(response)));
  }

  uploadPhotos(files: File[]): Observable<void> {
    const formData = new FormData();
    files.forEach(file => formData.append('files', file, file.name));
    return this.http.post<void>(`${this.apiBase}/photos`, formData);
  }

  getPhotoById(id: string): Observable<Photo | null> {
    return this.http.get<PhotoResponse>(`${this.apiBase}/photos/${id}`).pipe(
      map((dto) => this.mapPhoto(dto)),
      catchError(() => of(null))
    );
  }

  getPhotoMetadata(id: string): Observable<PhotoMetadata | null> {
    return this.http
      .get<PhotoMetadata | null>(`${this.apiBase}/photos/${id}/metadata`)
      .pipe(catchError(() => of(null)));
  }

  getAlbumComments(albumId: string): Observable<PagedModel<AlbumComment>> {
    return this.http
      .get<PagedModel<AlbumComment>>(`${this.apiBase}/albums/${albumId}/comments`)
      .pipe(
        catchError(() =>
          of({
            page: 1,
            pageSize: 0,
            total: 0,
            items: [],
          }),
        ),
      );
  }

  createAlbumComment(albumId: string, comment: string): Observable<AlbumComment> {
    return this.http.post<AlbumComment>(`${this.apiBase}/albums/${albumId}/comments`, { comment });
  }

  updateAlbumCommentVisibility(albumId: string, commentId: string, hidden: boolean): Observable<AlbumComment> {
    return this.http.patch<AlbumComment>(
      `${this.apiBase}/albums/${albumId}/comments/${commentId}/visibility`,
      { hidden },
    );
  }

  getPhotoComments(photoId: string): Observable<PhotoComment[]> {
    return this.http
      .get<PhotoComment[]>(`${this.apiBase}/photos/${photoId}/comments`)
      .pipe(catchError(() => of([])));
  }

  createPhotoComment(photoId: string, comment: string): Observable<PhotoComment> {
    return this.http.post<PhotoComment>(`${this.apiBase}/photos/${photoId}/comments`, { comment });
  }

  updatePhotoComment(photoId: string, comment: string | null): Observable<PhotoMetadata | null> {
    return this.http
      .patch<PhotoMetadata | null>(`${this.apiBase}/photos/${photoId}/metadata/comment`, { comment })
      .pipe(
        catchError(() => of(null))
      );
  }

  getThumbnailPath(photo: Photo): string {
    if (photo.hash) {
      return `${this.apiBase}/photos/thumbnail/${photo.hash}`;
    }

    return photo.path;
  }

  getDownloadPath(photo: Photo): string {
    return `${this.apiBase}/photos/file/${photo.id}`;
  }

  getAdjacentPhotos(id: string, albumId?: string): Observable<{ prevId: string | null; nextId: string | null }> {
    if (albumId) {
      return this.getAlbumPhotos(albumId, 1, 400).pipe(
        map((page) => {
          if (!page) return { prevId: null, nextId: null };
          const index = page.items.findIndex((photo) => photo.id === id);
          return this.resolveAdjacent(page.items.map(p => p.id), index);
        })
      );
    }

    let source$: Observable<string[]>;
    if (this.timelinePhotoIds) {
      source$ = of(this.timelinePhotoIds);
    } else {
      source$ = this.getTimeline().pipe(
        map(() => this.timelinePhotoIds || [])
      );
    }

    return source$.pipe(
      map(ids => {
        const index = ids.indexOf(id);
        return this.resolveAdjacent(ids, index);
      })
    );
  }

  private resolveAdjacent(ids: string[], index: number) {
    if (index === -1) return { prevId: null, nextId: null };
    const prevId = index > 0 ? ids[index - 1] : null;
    const nextId = index >= 0 && index < ids.length - 1 ? ids[index + 1] : null;
    return { prevId, nextId };
  }

  getTimeline(page: number = 1, pageSize: number = 10): Observable<GroupedPhotos[]> {
    if (page === 1 && this.timelineCache) {
      return of(this.timelineCache);
    }

    return this.http
      .get<{ title: string; photos: PagedModel<PhotoResponse> }[]>(`${this.apiBase}/photos/timeline/${page}/${pageSize}`)
      .pipe(
        map((groups) =>
          groups.map((g) => ({
            title: g.title,
            photos: this.mapPhotoPage(g.photos),
          }))
        ),
        tap(groups => {
          if (page === 1) {
            this.timelineCache = groups;
            this.timelinePhotoIds = groups.flatMap(g => g.photos.items.map(p => p.id));
          } else {
            if (this.timelineCache) {
              this.timelineCache.push(...groups);
            }
            if (this.timelinePhotoIds) {
              this.timelinePhotoIds.push(...groups.flatMap(g => g.photos.items.map(p => p.id)));
            }
          }
        }),
        catchError(() => of([]))
      );
  }

  getTimelineYears(): Observable<string[]> {
    return this.http.get<string[]>(`${this.apiBase}/photos/timeline/years`).pipe(
      catchError(() => of([]))
    );
  }

  getTimelineYearOffset(year: string): Observable<number> {
    return this.http.get<number>(`${this.apiBase}/photos/timeline/year-offset/${year}`).pipe(
      catchError(() => of(0))
    );
  }

  getGroupedPhotos(
    groupIndex: number,
    pageInGroup: number = 1,
    pageSize: number = 100
  ): Observable<GroupedPhotos | null> {
    return this.getTimeline().pipe(
      map((groups) => {
        if (groupIndex >= groups.length) {
          return null;
        }
        const target = groups[groupIndex];
        const start = (pageInGroup - 1) * pageSize;
        const items = target.photos.items.slice(start, start + pageSize);
        return {
          title: target.title,
          photos: {
            ...target.photos,
            page: pageInGroup,
            pageSize,
            items,
          },
        };
      })
    );
  }

  getAlbums(page = 1, pageSize = 12): Observable<PagedAlbumsModel> {
    return this.http
      .get<PagedModel<AlbumModel>>(`${this.apiBase}/albums/${page}/${pageSize}`)
      .pipe(
        map((response) => ({
          page: response.page,
          pageSize: response.pageSize,
          total: response.total,
          items: response.items.map((dto) => this.mapAlbum(dto)),
        }))
      );
  }

  createAlbum(album: Partial<AlbumModel>): Observable<Album> {
    return this.http.post<AlbumModel>(`${this.apiBase}/albums`, album).pipe(
      map(dto => this.mapAlbum(dto))
    );
  }

  deleteAlbum(id: string): Observable<void> {
    return this.http.delete(`${this.apiBase}/albums/${id}`, { responseType: 'text' }).pipe(
      map(() => undefined)
    );
  }

  updateAlbum(album: Partial<AlbumModel>): Observable<Album> {
    return this.http.put<AlbumModel>(`${this.apiBase}/albums`, album).pipe(
      map(dto => this.mapAlbum(dto))
    );
  }

  getAlbumById(id: string): Observable<Album | null> {
    return this.http.get<AlbumModel>(`${this.apiBase}/albums/${id}`).pipe(
      switchMap((dto) => {
        const rules = JSON.parse(dto.rulesJson || '{ "photoIds": [] }');

        return this.getAlbumPhotos(id, 1, 100).pipe(
          map((photos) => ({
            ...this.mapAlbum(dto),
            photos: photos || { page: 1, pageSize: 100, total: 0, items: [] },
          }))
        );
      }
      ),
      catchError(() => of(null))
    );
  }

  getAlbumPhotos(albumId: string, page = 1, pageSize = 20): Observable<PagedPhotos | null> {
    return this.http
      .get<PagedModel<PhotoResponse>>(`${this.apiBase}/albums/${albumId}/photos/${page}/${pageSize}`)
      .pipe(
        map((response) => this.mapPhotoPage(response)),
        catchError(() => of(null))
      );
  }

  private mapPhotoPage(response: PagedModel<PhotoResponse>): PagedPhotos {
    return {
      page: response.page,
      pageSize: response.pageSize,
      total: response.total,
      items: response.items.map((dto) => this.mapPhoto(dto)),
    };
  }

  private mapPhoto(dto: PhotoResponse): Photo {
    return {
      id: dto.id,
      path: dto.path,
      name: dto.name,
      format: dto.format ?? undefined,
      hash: dto.hash ?? undefined,
      size: dto.size ?? undefined,
      createdAt: this.toDate(dto.createdAt),
      updatedAt: this.toDate(dto.updatedAt),
      dateImported: this.toDate(dto.dateImported),
      dateTaken: this.toDate(dto.dateTaken),
      thumbnailPath: dto.thumbnailPath ?? undefined,
      thumbnailOptimized: dto.thumbnailOptimized ?? undefined,
      metadataExtracted: dto.metadataExtracted ?? undefined,
      isRaw: dto.isRaw ?? undefined,
      width: dto.width ?? undefined,
      height: dto.height ?? undefined,
      thumbnailWidth: dto.thumbnailWidth ?? undefined,
      thumbnailHeight: dto.thumbnailHeight ?? undefined,
    };
  }

  private mapAlbum(dto: AlbumModel): Album {
    return {
      id: dto.id,
      parentId: dto.parentId ?? undefined,
      name: dto.name,
      createDate: this.toDate(dto.createDate),
      description: dto.description ?? undefined,
      category: dto.category ?? undefined,
      kind: (dto.kind === 'smart' ? 'smart' : 'manual') as Album['kind'],
      rulesJson: dto.rulesJson ?? undefined,
      thumbnailHash: dto.thumbnailHash ?? undefined,
      sortOrder: dto.sortOrder ?? 0,
      imageCount: dto.imageCount ?? undefined,
    };
  }


  getPhotosWithGps(page = 1, pageSize = 100): Observable<{ page: number, pageSize: number, total: number, items: PhotoLoc[] }> {
    return this.http
      .get<PagedModel<PhotoLocResponse>>(`${this.apiBase}/photos/with-gps/${page}/${pageSize}`)
      .pipe(
        map((response) => ({
          page: response.page,
          pageSize: response.pageSize,
          total: response.total ?? 0,
          items: response.items.map((dto) => this.mapPhotoLoc(dto)),
        }))
      );
  }

  private mapPhotoLoc(dto: PhotoLocResponse): PhotoLoc {
    const photo = this.mapPhoto(dto);
    return {
      ...photo,
      lat: dto.lat,
      lon: dto.lon
    };
  }

  private toDate(value?: string | null): Date | undefined {
    if (!value) {
      return undefined;
    }
    const parsed = new Date(value);
    return Number.isNaN(parsed.getTime()) ? undefined : parsed;
  }
}
