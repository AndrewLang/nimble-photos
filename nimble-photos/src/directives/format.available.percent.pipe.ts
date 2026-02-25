import { Pipe, PipeTransform } from '@angular/core';

import { Formatter } from '../models/formatters';

@Pipe({
    name: 'formatAvailablePercent',
})
export class FormatAvailablePercentPipe implements PipeTransform {
    transform(availableBytes?: number, totalBytes?: number): string {
        return Formatter.formatAvailablePercent(availableBytes ?? 0, totalBytes ?? 0);
    }
}
