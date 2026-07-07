import { translations } from './translations';

export type Locale = 'en' | 'bn' | 'ar';

let locale = $state<Locale>('en');
let direction = $derived(locale === 'ar' ? 'rtl' : 'ltr');

function t(key: string): string {
  return translations[locale][key] ?? key;
}

function setLocale(l: Locale): void {
  locale = l;
  document.documentElement.dir = direction;
  document.documentElement.lang = l;
}

export function getI18n() {
  return {
    get locale() {
      return locale;
    },
    get direction() {
      return direction;
    },
    t,
    setLocale,
  };
}
