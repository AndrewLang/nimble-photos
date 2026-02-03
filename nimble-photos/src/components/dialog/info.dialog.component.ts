import { Component, Input } from '@angular/core';

@Component({
    selector: 'mtx-info-dialog',
    standalone: true,
    template: `
    <div class="flex flex-col items-center text-center gap-4">
      <div class="h-20 w-20 rounded-full bg-indigo-500/10 flex items-center justify-center text-indigo-500 mb-2">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>
      <h2 class="text-xl font-bold text-white">Feature Coming Soon</h2>
      <p class="text-slate-400 text-sm leading-relaxed">
        We're working hard to bring this feature to Nimble Photos. Stay tuned for updates!
      </p>
    </div>
  `
})
export class InfoDialog { }
