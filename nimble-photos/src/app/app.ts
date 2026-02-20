import { Component, OnInit, inject } from '@angular/core';
import { Router, RouterModule } from '@angular/router';
import { catchError, of } from 'rxjs';
import { AuthService } from '../services/auth.service';
import { HeaderComponent } from '../components/header/header.component';

@Component({
  selector: 'mtx-root',
  imports: [RouterModule, HeaderComponent],
  templateUrl: './app.html'
})
export class App implements OnInit {
  private readonly authService = inject(AuthService);
  private readonly router = inject(Router);

  ngOnInit(): void {
    this.authService.getRegistrationStatus()
      .pipe(catchError(() => of(null)))
      .subscribe(status => {
        if (!status || status.initialized) {
          return;
        }

        const currentUrl = this.router.url;
        if (!currentUrl.startsWith('/setup')) {
          this.router.navigate(['/setup']);
        }
      });
  }
}
