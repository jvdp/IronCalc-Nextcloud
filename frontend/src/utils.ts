// Locale & language helpers

export function getDefaultUILocale(): string {
  const lang = navigator.language || navigator.languages[0] || "en-US";
  if (lang.startsWith("es")) {
    return "es-ES";
  } else if (lang.startsWith("fr")) {
    return "fr-FR";
  } else if (lang.startsWith("de")) {
    return "de-DE";
  } else if (lang === "en-GB") {
    return "en-GB";
  } else if (lang.startsWith("it")) {
    return "it-IT";
  }

  return "en-US";
}

export function getShortLocaleCode(longCode: string): string {
  switch (longCode) {
    case "es-ES": {
      return "es";
    }
    case "fr-FR": {
      return "fr";
    }
    case "de-DE": {
      return "de";
    }
    case "it-IT": {
      return "it";
    }
    default: {
      return "en";
    }
  }
}

export function getLanguageFromLocale(locale: string): string {
  return locale.split("-")[0];
}

export function getTimezone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone;
  } catch {
    return "UTC";
  }
}

// Nextcloud URL helpers

export type Location = {
  fileId: string;
  filePath: string;
  fileName: string;
  dir: string;
};

const APP_PATH_RE = /^(\/apps\/app_api\/embedded\/ironcalc\/ironcalc\/)(.+)/;

export function getLocation(): Location | undefined {
  const fileId = new URLSearchParams(window.location.search)
    .get("fileIds")
    ?.split(",")[0];
  const match = window.location.pathname.match(APP_PATH_RE);
  const filePath = match ? decodeURIComponent(match[2]) : undefined;
  if (!fileId || !filePath) return undefined;
  return {
    fileId,
    filePath,
    fileName: filePath.split("/").at(-1) ?? "",
    dir: filePath.substring(0, filePath.lastIndexOf("/")),
  };
}

export function buildFileUrl(path: string, fileId: number | string): string {
  return `/apps/app_api/embedded/ironcalc/ironcalc/${path}?fileIds=${fileId}`;
}

export function downloadFile(filePath: string): void {
  window.open(
    `/remote.php/webdav/${filePath.split("/").map(encodeURIComponent).join("/")}`,
    "_blank",
  );
}

export function navigateToFolder(filePath?: string): void {
  if (filePath) {
    const dir = `/${filePath.substring(0, filePath.lastIndexOf("/"))}`;
    window.location.href = `/index.php/apps/files/files?dir=${encodeURIComponent(dir)}`;
  } else {
    window.location.href = "/index.php/apps/files";
  }
}

// Filename helpers

export function ensureExtension(name: string): string {
  return name.endsWith(".xlsx") ? name : `${name}.xlsx`;
}
