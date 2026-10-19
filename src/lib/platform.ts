const ua = navigator.userAgent;

function detect(): "macos" | "windows" | "linux" | "other" {
    if (/Macintosh|Mac OS X/.test(ua)) return "macos";
    if (/Windows NT/.test(ua)) return "windows";
    if (/Linux|X11/.test(ua)) return "linux";
    return "other";
}

export const platform = detect();
