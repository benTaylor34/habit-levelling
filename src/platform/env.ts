/** True inside the Android app (its web view reports "Android" in the user agent). */
export const isAndroid = /android/i.test(navigator.userAgent);
