/**
 * Announce a message to screen readers via a live region.
 * This module must be used together with <Announcements /> component
 * which provides the DOM elements for the live region.
 */

type Priority = 'polite' | 'assertive';

let politeEl: HTMLElement | null = null;
let assertiveEl: HTMLElement | null = null;
let politeTimer: ReturnType<typeof setTimeout> | undefined;
let assertiveTimer: ReturnType<typeof setTimeout> | undefined;

export function initAnnouncements(): void {
  politeEl = document.getElementById('a11y-polite');
  assertiveEl = document.getElementById('a11y-assertive');
}

export function announce(message: string, priority: Priority = 'polite'): void {
  const el = priority === 'assertive' ? assertiveEl : politeEl;
  const timer = priority === 'assertive' ? assertiveTimer : politeTimer;
  const setTimer = (t: ReturnType<typeof setTimeout> | undefined) => {
    if (priority === 'assertive') {
      assertiveTimer = t;
    } else {
      politeTimer = t;
    }
  };

  if (!el) return;
  clearTimeout(timer);
  el.textContent = '';
  requestAnimationFrame(() => {
    el.textContent = message;
    setTimer(setTimeout(() => {
      el.textContent = '';
    }, 6000));
  });
}
