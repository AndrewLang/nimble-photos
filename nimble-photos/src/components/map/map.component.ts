import { CommonModule } from '@angular/common';
import { Component, ElementRef, OnDestroy, OnInit, signal, ViewChild } from '@angular/core';
import { Router, RouterModule } from '@angular/router';
import * as L from 'leaflet';
import { first } from 'rxjs';
import { Photo, PhotoLoc } from '../../models/photo';
import { PhotoService } from '../../services/photo.service';

@Component({
  selector: 'mtx-map',
  standalone: true,
  imports: [CommonModule, RouterModule],
  templateUrl: './map.component.html',
  host: {
    class: 'block flex-1 min-h-0',
  }
})
export class MapComponent implements OnInit, OnDestroy {
  @ViewChild('mapContainer', { static: true }) mapContainer!: ElementRef;

  private map?: L.Map;
  readonly photos = signal<PhotoLoc[]>([]);
  readonly loading = signal(true);
  readonly hasGpsData = signal(false);

  constructor(
    private readonly photoService: PhotoService,
    private readonly router: Router
  ) { }

  ngOnInit(): void {
    delete (L.Icon.Default.prototype as any)._getIconUrl;
    L.Icon.Default.mergeOptions({
      iconRetinaUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon-2x.png',
      iconUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-icon.png',
      shadowUrl: 'https://unpkg.com/leaflet@1.9.4/dist/images/marker-shadow.png',
    });

    this.fetchPhotos();
    // this.loading.set(false);
  }

  ngOnDestroy(): void {
    if (this.map) {
      this.map.remove();
    }
  }

  private fetchPhotos(): void {
    this.photoService.getPhotosWithGps(1, 1000).pipe(first()).subscribe({
      next: (paged) => {
        const photosWithGps = paged.items;
        this.hasGpsData.set(photosWithGps.length > 0);
        this.photos.set(photosWithGps);

        this.initMap();
        this.loading.set(false);
      },
      error: (error) => {
        console.error('MapComponent: Failed to load GPS photos', error);
        this.initMap();
        this.loading.set(false);
      }
    });
  }

  private initMap(): void {
    if (this.map) {
      return;
    }

    try {
      this.map = L.map(this.mapContainer.nativeElement, {
        center: [20, 0],
        zoom: 3,
        zoomControl: false,
        attributionControl: false
      });


      L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', {
        maxZoom: 20
      }).addTo(this.map);


      L.control.zoom({
        position: 'bottomright'
      }).addTo(this.map);

      this.addMarkers();

      setTimeout(() => {
        if (this.map) {
          this.map.invalidateSize();
        }
      }, 100);
    } catch (error) {
      console.error('MapComponent: Error initializing map', error);
    }
  }

  private addMarkers(): void {
    if (!this.map) return;

    const groups = new Map<string, PhotoLoc[]>();
    this.photos().forEach(photo => {
      const lat = photo.lat;
      const lng = photo.lon;

      const key = `${lat}_${lng}`;
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key)!.push(photo);
    });

    const markerGroup = L.featureGroup();

    groups.forEach((groupPhotos) => {
      const firstPhoto = groupPhotos[0];
      const lat = firstPhoto.lat;
      const lng = firstPhoto.lon;
      const count = groupPhotos.length;
      const thumbnail = this.photoThumbnail(firstPhoto);
      const iconHtml = count > 1
        ? `<div class="cluster-marker">
                     <div style="width: 40px; height: 40px; overflow: hidden; border-radius: 50%; border: 2px solid white; box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.3);">
                       <img src="${thumbnail}" class="marker-thumbnail" style="width: 100%; height: 100%; object-fit: cover;" />
                     </div>
                     <span class="marker-count">${count}</span>
                   </div>`
        : `<div style="width: 40px; height: 40px; overflow: hidden; border-radius: 50%; border: 2px solid white;">
                     <img src="${thumbnail}" class="marker-thumbnail" style="width: 100%; height: 100%; object-fit: cover;" />
                   </div>`;

      const customIcon = L.divIcon({
        className: 'photo-marker',
        html: iconHtml,
        iconSize: [40, 40],
        iconAnchor: [20, 20]
      });

      const latStr = `${Math.abs(lat).toFixed(4)}deg${lat >= 0 ? 'N' : 'S'}`;
      const lngStr = `${Math.abs(lng).toFixed(4)}deg${lng >= 0 ? 'E' : 'W'}`;

      const dates = groupPhotos
        .map(p => (p.dateTaken ?? p.createdAt)?.valueOf())
        .filter((value): value is number => value !== undefined)
        .sort((a, b) => a - b);
      const minDate = dates.length ? new Date(dates[0]) : null;
      const maxDate = dates.length ? new Date(dates[dates.length - 1]) : null;
      const dateStr = minDate
        ? `${minDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}${count > 1 && maxDate ? ` - ${maxDate.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}` : ''}`
        : 'Date unknown';

      const popupContent = `
              <div class="map-grid-popup">
                <div class="map-grid-header">
                  <h3>${count} photo${count > 1 ? 's' : ''}</h3>
                  <span>Click image to view details</span>
                </div>
                <div class="map-grid">
                  ${groupPhotos.slice(0, 6).map(p => `
                    <img src="${this.photoThumbnail(p)}" alt="${p.name}" data-id="${p.id}" class="popup-grid-img" />
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

      marker.on('mouseover', () => marker.openPopup());

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

  private photoThumbnail(photo: Photo): string {
    return this.photoService.getThumbnailPath(photo);
  }
}
