export class SvgIcon {
  static readonly defaultViewBox = '0 0 24 24';
  constructor(
    public readonly name: string,
    public readonly viewBox: string,
    public readonly paths: string[]
  ) { }

  private static readonly registry = new Map<string, SvgIcon>();

  private static register(icon: SvgIcon): SvgIcon {
    SvgIcon.registry.set(icon.name, icon);
    return icon;
  }

  static getIcon(name: string): SvgIcon | null {
    return SvgIcon.registry.get(name) ?? null;
  }

  static get(name: string): string {
    const icon = SvgIcon.getIcon(name);
    if (!icon)
      return "";

    const content = icon.paths.map((part) =>
      part.trim().startsWith("<")
        ? (part.trim().endsWith("/>") || part.trim().includes("</")
          ? part
          : `${part} />`)
        : `<path d="${part}" />`
    ).join("\n  ");
    return `
    <svg viewBox="${icon.viewBox}" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      ${content}
    </svg>`;
  }

  static names(): string[] {
    return Array.from(SvgIcon.registry.keys());
  }

  static readonly settings = SvgIcon.register(
    new SvgIcon('settings', SvgIcon.defaultViewBox, [
      'M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7Z',
      'M19.4 15a1.8 1.8 0 0 0 .36 2l.06.07a2 2 0 0 1-1.42 3.42h-.18a1.8 1.8 0 0 0-1.71 1.24l-.05.12a2 2 0 0 1-3.7 0l-.05-.12a1.8 1.8 0 0 0-1.71-1.24h-.18a2 2 0 0 1-1.42-3.42l.06-.07a1.8 1.8 0 0 0 .36-2l-.07-.12a1.8 1.8 0 0 0-1.59-.99H4a2 2 0 0 1 0-4h.12a1.8 1.8 0 0 0 1.59-.99l.07-.12a1.8 1.8 0 0 0-.36-2l-.06-.07A2 2 0 0 1 6.8 3.4h.18a1.8 1.8 0 0 0 1.71-1.24l.05-.12a2 2 0 0 1 3.7 0l.05.12a1.8 1.8 0 0 0 1.71 1.24h.18a2 2 0 0 1 1.42 3.42l-.06.07a1.8 1.8 0 0 0-.36 2l.07.12a1.8 1.8 0 0 0 1.59.99H20a2 2 0 1 1 0 4h-.12a1.8 1.8 0 0 0-1.59.99l-.07.12Z',
    ])
  );

  static readonly search = SvgIcon.register(
    new SvgIcon('search', SvgIcon.defaultViewBox, [
      'M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z',
    ])
  );

  static readonly chevronRight = SvgIcon.register(
    new SvgIcon('chevronRight', SvgIcon.defaultViewBox, [
      'M9 6l6 6-6 6',
    ])
  );

  static readonly userCircle = SvgIcon.register(
    new SvgIcon('userCircle', SvgIcon.defaultViewBox, [
      'M17.982 18.725A7.488 7.488 0 0012 15.75a7.488 7.488 0 00-5.982 2.975m11.963 0a9 9 0 10-11.963 0m11.963 0A8.966 8.966 0 0112 21a8.966 8.966 0 01-5.982-2.275M15 9.75a3 3 0 11-6 0 3 3 0 016 0z',
    ])
  );

  static readonly errorCircle = SvgIcon.register(
    new SvgIcon('errorCircle', SvgIcon.defaultViewBox, [
      'M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z',
    ])
  );

  static readonly settingItem = SvgIcon.register(
    new SvgIcon('settingItem', SvgIcon.defaultViewBox, [
      'M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.041.147.087.218.137.319.231.758.241 1.082.025l1.114-.738a1.125 1.125 0 011.507.248l1.296 2.246a1.125 1.125 0 01-.248 1.507l-1.114.738a1.125 1.125 0 00-.025 1.082c.05.07.096.144.137.217.184.332.496.582.87.645l1.281.213c.542.09.94.56.94 1.11v2.593c0 .55-.398 1.02-.94 1.11l-1.281.213a1.125 1.125 0 00-.87.645c-.041.074-.087.147-.137.218-.231.319-.241.758-.025 1.082l.738 1.114a1.125 1.125 0 01-.248 1.507l-2.246 1.296a1.125 1.125 0 01-1.507-.248l-.738-1.114a1.125 1.125 0 00-1.082-.025c-.07.05-.144.096-.217.137-.332.184-.582.496-.645.87l-.213 1.281c-.09.542-.56.94-1.11.94h-2.593c-.55 0-1.02-.398-1.11-.94l-.213-1.281a1.125 1.125 0 00-.645-.87c-.074-.041-.147-.087-.218-.137-.319-.231-.758-.241-1.082-.025l-1.114.738a1.125 1.125 0 01-1.507-.248l-1.296-2.246a1.125 1.125 0 01.248-1.507l1.114-.738a1.125 1.125 0 00.025-1.082c-.05-.07-.096-.144-.137-.217a1.125 1.125 0 00-.87-.645l-1.281-.213A1.125 1.125 0 013 13.068v-2.593c0-.55.398-1.02.94-1.11l1.281-.213a1.125 1.125 0 00.87-.645c.041-.074.087-.147.137-.218.231-.319.241-.758.025-1.082L5.515 6.13a1.125 1.125 0 01.248-1.507l2.246-1.296a1.125 1.125 0 011.507.248l.738 1.114a1.125 1.125 0 001.082.025c.07-.05.144-.096.217-.137.332-.184.582-.496.645-.87l.213-1.281z',
      'M15 12a3 3 0 11-6 0 3 3 0 016 0z',
    ])
  );

  static readonly sectionGeneral = SvgIcon.register(
    new SvgIcon('sectionGeneral', SvgIcon.defaultViewBox, [
      'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
    ])
  );

  static readonly sectionExperience = SvgIcon.register(
    new SvgIcon('sectionExperience', SvgIcon.defaultViewBox, [
      'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z',
    ])
  );

  static readonly sectionNotifications = SvgIcon.register(
    new SvgIcon('sectionNotifications', SvgIcon.defaultViewBox, [
      'M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9',
    ])
  );

  static readonly sectionStorage = SvgIcon.register(
    new SvgIcon('sectionStorage', SvgIcon.defaultViewBox, [
      'M5 4h14a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2Z',
      'M5 14h14a2 2 0 0 1 2 2v2a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-2a2 2 0 0 1 2-2Z',
      'M9.2 7a1.2 1.2 0 1 1-2.4 0a1.2 1.2 0 1 1 2.4 0Z',
      'M9.2 17a1.2 1.2 0 1 1-2.4 0a1.2 1.2 0 1 1 2.4 0Z',
    ])
  );

  static readonly sectionPhotoManage = SvgIcon.register(
    new SvgIcon('sectionPhotoManage', SvgIcon.defaultViewBox, [
      'M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z',
    ])
  );

  static readonly sectionDefault = SvgIcon.register(
    new SvgIcon('sectionDefault', SvgIcon.defaultViewBox, [
      'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z',
      'M15 12a3 3 0 11-6 0 3 3 0 016 0z',
    ])
  );

  static readonly checkSolid = SvgIcon.register(
    new SvgIcon('checkSolid', SvgIcon.defaultViewBox, [
      'M5 12l4 4L19 6',
    ])
  );

  static readonly sparkle = SvgIcon.register(
    new SvgIcon('sparkle', SvgIcon.defaultViewBox, [
      'M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-7.714 2.143L11 21l-2.286-6.857L1 12l7.714-2.143L11 3z',
    ])
  );

  static readonly closeSolid = SvgIcon.register(
    new SvgIcon('closeSolid', '0 0 20 20', [
      'M5 5l10 10',
      'M15 5l-10 10',
    ])
  );

  static readonly plus = SvgIcon.register(
    new SvgIcon('plus', SvgIcon.defaultViewBox, [
      'M12 4v16m8-8H4',
    ])
  );

  static readonly folderPlus = SvgIcon.register(
    new SvgIcon('folderPlus', SvgIcon.defaultViewBox, [
      'M9 13h6m-3-3v6m-9 1V7a2 2 0 0 1 2-2h6l2 2h6a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
    ])
  );

  static readonly tag = SvgIcon.register(
    new SvgIcon('tag', SvgIcon.defaultViewBox, [
      'M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 0 1 0 2.828l-7 7a2 2 0 0 1-2.828 0l-7-7A1.994 1.994 0 0 1 3 12V7a4 4 0 0 1 4-4z',
    ])
  );

  static readonly download = SvgIcon.register(
    new SvgIcon('download', SvgIcon.defaultViewBox, [
      'M4 16v1a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-1m-4-4l-4 4m0 0l-4-4m4 4V4',
    ])
  );

  static readonly user = SvgIcon.register(
    new SvgIcon('user', SvgIcon.defaultViewBox, [
      'M16 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0z',
      'M12 14a7 7 0 0 0-7 7h14a7 7 0 0 0-7-7z',
    ])
  );

  static readonly menu = SvgIcon.register(
    new SvgIcon('menu', SvgIcon.defaultViewBox, [
      'M4 6h16M4 12h16M4 18h16',
    ])
  );

  static readonly close = SvgIcon.register(
    new SvgIcon('close', SvgIcon.defaultViewBox, [
      'M6 18L18 6M6 6l12 12',
    ])
  );

  static readonly imagePlaceholder = SvgIcon.register(
    new SvgIcon('imagePlaceholder', SvgIcon.defaultViewBox, [
      'M4 16l4.586-4.586a2 2 0 0 1 2.828 0L16 16m-2-2l1.586-1.586a2 2 0 0 1 2.828 0L20 14m-6-6h.01M6 20h12a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2z',
    ])
  );

  static readonly trash = SvgIcon.register(
    new SvgIcon('trash', SvgIcon.defaultViewBox, [
      'M19 7l-.867 12.142A2 2 0 0 1 16.138 21H7.862a2 2 0 0 1-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v3M4 7h16',
    ])
  );

  static readonly arrowRight = SvgIcon.register(
    new SvgIcon('arrowRight', SvgIcon.defaultViewBox, [
      'M14 5l7 7m0 0l-7 7m7-7H3',
    ])
  );

  static readonly lock = SvgIcon.register(
    new SvgIcon('lock', SvgIcon.defaultViewBox, [
      'M12 15v2m-6 4h12a2 2 0 0 0 2-2v-6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2zm10-10V7a4 4 0 0 0-8 0v4h8z',
    ])
  );

  static readonly send = SvgIcon.register(
    new SvgIcon('send', SvgIcon.defaultViewBox, [
      'M4 12h14',
      'M13 5l7 7-7 7',
    ])
  );

  static readonly arrowLeft = SvgIcon.register(
    new SvgIcon('arrowLeft', SvgIcon.defaultViewBox, [
      'M10 19l-7-7 7-7',
    ])
  );

  static readonly checkCircle = SvgIcon.register(
    new SvgIcon('checkCircle', SvgIcon.defaultViewBox, [
      'M9 12l2 2 4-4m6 2a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
    ])
  );

  static readonly errorCircleOutline = SvgIcon.register(
    new SvgIcon('errorCircleOutline', SvgIcon.defaultViewBox, [
      'M12 8v4m0 4h.01M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
    ])
  );

  static readonly mail = SvgIcon.register(
    new SvgIcon('mail', SvgIcon.defaultViewBox, [
      'M3 8l7.89 5.26a2 2 0 0 0 2.22 0L21 8M5 19h14a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2z',
    ])
  );

  static readonly eye = SvgIcon.register(
    new SvgIcon('eye', SvgIcon.defaultViewBox, [
      'M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z',
      'M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z',
    ])
  );

  static readonly eyeOff = SvgIcon.register(
    new SvgIcon('eyeOff', SvgIcon.defaultViewBox, [
      'M13.875 18.825A10.05 10.05 0 0 1 12 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 0 1 1.563-3.029',
      'M9.878 9.878a3 3 0 1 1 4.243 4.243',
      'M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l18 18',
    ])
  );

  static readonly checkOutline = SvgIcon.register(
    new SvgIcon('checkOutline', SvgIcon.defaultViewBox, [
      'M5 13l4 4L19 7',
    ])
  );

  static readonly infoCircle = SvgIcon.register(
    new SvgIcon('infoCircle', SvgIcon.defaultViewBox, [
      'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
    ])
  );

  static readonly mapPin = SvgIcon.register(
    new SvgIcon('mapPin', SvgIcon.defaultViewBox, [
      'M17.657 16.657L13.414 20.9a1.998 1.998 0 0 1-2.827 0l-4.244-4.243a8 8 0 1 1 11.314 0z',
      'M15 11a3 3 0 1 1-6 0 3 3 0 0 1 6 0z',
    ])
  );

  static readonly mapPinSolid = SvgIcon.register(
    new SvgIcon('mapPinSolid', SvgIcon.defaultViewBox, [
      'M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7zm0 9.5c-1.38 0-2.5-1.12-2.5-2.5s1.12-2.5 2.5-2.5 2.5 1.12 2.5 2.5-1.12 2.5-2.5 2.5z',
    ])
  );

  static readonly calendar = SvgIcon.register(
    new SvgIcon('calendar', SvgIcon.defaultViewBox, [
      'M19 4h-1V2h-2v2H8V2H6v2H5c-1.11 0-1.99.9-1.99 2L3 20c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 16H5V10h14v10zm0-12H5V6h14v2z',
    ])
  );

  static readonly chevronLeftBold = SvgIcon.register(
    new SvgIcon('chevronLeftBold', SvgIcon.defaultViewBox, [
      'M15 19l-7-7 7-7',
    ])
  );

  static readonly chevronRightBold = SvgIcon.register(
    new SvgIcon('chevronRightBold', SvgIcon.defaultViewBox, [
      'M9 5l7 7-7 7',
    ])
  );

  static readonly chevronDownSolid = SvgIcon.register(
    new SvgIcon('chevronDownSolid', '0 0 20 20', [
      '<path fill="currentColor" fill-rule="evenodd" d="M5.23 7.21a.75.75 0 0 1 1.06.02L10 10.586l3.71-3.356a.75.75 0 0 1 1.02 1.097l-4 3.615a.75.75 0 0 1-1.02 0l-4-3.615a.75.75 0 0 1 .02-1.057z" clip-rule="evenodd" />',
    ])
  );

  static readonly panelRightClose = SvgIcon.register(
    new SvgIcon('panelRightClose', SvgIcon.defaultViewBox, [
      'M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5z',
      'M15 3v18',
      'm9 9 3 3-3 3',
    ])
  );

  static readonly panelLeftClose = SvgIcon.register(
    new SvgIcon('panelLeftClose', SvgIcon.defaultViewBox, [
      'M3 5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5z',
      'M9 3v18',
      'm15 9-3 3 3 3',
    ])
  );

}
