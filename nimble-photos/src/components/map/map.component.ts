import { Component, OnInit, signal, effect, ElementRef, ViewChild, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Router } from '@angular/router';
import { first } from 'rxjs';
import * as L from 'leaflet';
import { PhotoService } from '../../services/photo.service';
import { Photo } from '../../models/photo.model';

@Component({
    selector: 'mtx-map',
    standalone: true,
    imports: [CommonModule, RouterModule],
    templateUrl: './map.component.html',
    styles: [`
    :host {
      display: block;
      height: 100%;
      width: 100%;
    }
    #map {
      height: 100%;
      width: 100%;
      z-index: 10;
      background: #020617;
    }
    .photo-marker {
      background: none;
      border: none;
    }
    .marker-thumbnail {
      width: 40px;
      height: 40px;
      border-radius: 50%;
      border: 2px solid white;
      box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
      object-fit: cover;
      display: block;
      transition: transform 0.2s;
    }
    .cluster-marker {
      position: relative;
    }
    .marker-count {
      position: absolute;
      top: -6px;
      right: -6px;
      background: #4f46e5;
      color: white;
      font-size: 10px;
      font-weight: 800;
      width: 20px;
      height: 20px;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      border: 2px solid #0f172a;
      box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
      z-index: 30;
    }
    .marker-thumbnail:hover {
      transform: scale(1.1);
      z-index: 1000;
      border-color: #6366f1;
    }
    ::ng-deep .leaflet-popup-content-wrapper {
      background: #0f172a !important;
      color: white !important;
      backdrop-filter: blur(16px);
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 1.5rem !important;
      padding: 0 !important;
      overflow: hidden;
      box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5) !important;
    }
    ::ng-deep .leaflet-popup-content {
      margin: 0 !important;
      width: 320px !important;
    }
    ::ng-deep .leaflet-popup-tip {
      background: #0f172a !important;
    }
    ::ng-deep .map-grid-popup {
      padding: 1.5rem;
    }
    ::ng-deep .map-grid-header {
      display: flex;
      justify-content: space-between;
      align-items: baseline;
      margin-bottom: 1rem;
    }
    ::ng-deep .map-grid-header h3 {
      font-size: 1.15rem;
      font-weight: 700;
      color: white;
      margin: 0;
    }
    ::ng-deep .map-grid-header span {
      font-size: 0.75rem;
      color: rgba(255, 255, 255, 0.5);
      font-weight: 500;
    }
    ::ng-deep .map-grid {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 0.5rem;
      margin-bottom: 1.5rem;
    }
    ::ng-deep .map-grid img {
      width: 100%;
      aspect-ratio: 1;
      object-fit: cover;
      border-radius: 0.75rem;
      background: rgba(255, 255, 255, 0.05);
      transition: all 0.2s;
      cursor: pointer;
    }
    ::ng-deep .map-grid img:hover {
      transform: scale(1.05);
      filter: brightness(1.2);
    }
    ::ng-deep .map-footer {
      border-top: 1px solid rgba(255, 255, 255, 0.05);
      padding-top: 1rem;
    }
    ::ng-deep .footer-row {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      margin-bottom: 0.5rem;
      color: rgba(255, 255, 255, 0.6);
      font-size: 0.75rem;
    }
    ::ng-deep .footer-row svg {
      width: 14px;
      height: 14px;
      opacity: 0.5;
    }
  `],
    host: {
        class: 'block flex-1 min-h-0',
    }
})
export class MapComponent implements OnInit, OnDestroy {
    @ViewChild('mapContainer', { static: true }) mapContainer!: ElementRef;

    private map?: L.Map;
    readonly photos = signal<Photo[]>([]);
    readonly loading = signal(true);

    constructor(
        private readonly photoService: PhotoService,
        private readonly router: Router
    ) { }

    ngOnInit(): void {
        this.fetchPhotos();
    }

    ngOnDestroy(): void {
        if (this.map) {
            this.map.remove();
        }
    }

    private fetchPhotos(): void {
        // Fetch a large-ish batch to fill the map
        this.photoService.getPhotos(1, 100).pipe(first()).subscribe(paged => {
            const photosWithGps = paged.items.filter(p => p.metadata.lat !== undefined && p.metadata.lng !== undefined);
            this.photos.set(photosWithGps);
            this.initMap();
            this.loading.set(false);
        });
    }

    private initMap(): void {
        if (this.map) return;

        // Default view: Center on Europe/Global
        this.map = L.map('map', {
            center: [20, 0],
            zoom: 3,
            zoomControl: false,
            attributionControl: false
        });

        // Dark Mode Tiles
        L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', {
            maxZoom: 20
        }).addTo(this.map);

        L.control.zoom({
            position: 'bottomright'
        }).addTo(this.map);

        this.addMarkers();
    }

    private addMarkers(): void {
        if (!this.map) return;

        const groups = new Map<string, Photo[]>();
        this.photos().forEach(photo => {
            const key = `${photo.metadata.lat}_${photo.metadata.lng}`;
            if (!groups.has(key)) groups.set(key, []);
            groups.get(key)!.push(photo);
        });

        const markerGroup = L.featureGroup();

        groups.forEach((groupPhotos, key) => {
            const firstPhoto = groupPhotos[0];
            const count = groupPhotos.length;

            const iconHtml = count > 1
                ? `<div class="cluster-marker">
                     <div style="width: 40px; height: 40px; overflow: hidden; border-radius: 50%; border: 2px solid white; box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3);">
                       <img src="${firstPhoto.url.replace('w=900', 'w=100')}" class="marker-thumbnail" style="width: 100%; height: 100%; object-fit: cover;" />
                     </div>
                     <span class="marker-count">${count}</span>
                   </div>`
                : `<div style="width: 40px; height: 40px; overflow: hidden; border-radius: 50%; border: 2px solid white;">
                     <img src="${firstPhoto.url.replace('w=900', 'w=100')}" class="marker-thumbnail" style="width: 100%; height: 100%; object-fit: cover;" />
                   </div>`;

            const customIcon = L.divIcon({
                className: 'photo-marker',
                html: iconHtml,
                iconSize: [40, 40],
                iconAnchor: [20, 20]
            });

            const lat = firstPhoto.metadata.lat!;
            const lng = firstPhoto.metadata.lng!;
            const latStr = `${Math.abs(lat).toFixed(4)}°${lat >= 0 ? 'N' : 'S'}`;
            const lngStr = `${Math.abs(lng).toFixed(4)}°${lng >= 0 ? 'E' : 'W'}`;

            const dates = groupPhotos.map(p => p.dateCreated.getTime()).sort();
            const minDate = new Date(dates[0]);
            const maxDate = new Date(dates[dates.length - 1]);
            const dateStr = minDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }) +
                (count > 1 ? ` - ${maxDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}` : '');

            const popupContent = `
              <div class="map-grid-popup">
                <div class="map-grid-header">
                  <h3>${count} photos</h3>
                  <span>Click image to view details</span>
                </div>
                <div class="map-grid">
                  ${groupPhotos.slice(0, 6).map(p => `
                    <img src="${p.url.replace('w=900', 'w=100')}" alt="${p.title}" data-id="${p.id}" class="popup-grid-img" />
                  `).join('')}
                </div>
                <div class="map-footer">
                  <div class="footer-row">
                    <svg fill="currentColor" viewBox="0 0 24 24"><path d="M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7zm0 9.5c-1.38 0-2.5-1.12-2.5-2.5s1.12-2.5 2.5-2.5 2.5 1.12 2.5 2.5-1.12 2.5-2.5 2.5z"/></svg>
                    <span>${latStr}, ${lngStr}</span>
                  </div>
                  <div class="footer-row">
                    <svg fill="currentColor" viewBox="0 0 24 24"><path d="M19 4h-1V2h-2v2H8V2H6v2H5c-1.11 0-1.99.9-1.99 2L3 20c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 16H5V10h14v10zm0-12H5V6h14v2z"/></svg>
                    <span>${dateStr}</span>
                  </div>
                </div>
              </div>
            `;

            const marker = L.marker([lat, lng], { icon: customIcon })
                .bindPopup(popupContent, {
                    maxWidth: 320,
                    closeButton: false
                });

            marker.on('mouseover', (e) => {
                marker.openPopup();
            });

            marker.on('popupopen', () => {
                const gridImages = document.querySelectorAll('.popup-grid-img');
                gridImages.forEach(img => {
                    img.addEventListener('click', (e) => {
                        const id = (e.currentTarget as HTMLElement).getAttribute('data-id');
                        if (id) {
                            this.router.navigate(['/photo', id]);
                        }
                    });
                });
            });

            marker.addTo(markerGroup);
        });

        markerGroup.addTo(this.map);

        if (this.photos().length > 0) {
            this.map.fitBounds(markerGroup.getBounds(), { padding: [50, 50] });
        }
    }
}
