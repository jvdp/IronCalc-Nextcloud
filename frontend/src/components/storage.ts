import { Model } from "@ironcalc/workbook";

export function createModelWithSafeTimezone(name: string): Model {
  try {
    const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
    return new Model(name, "en", tz);
  } catch {
    console.warn("Failed to get timezone, defaulting to UTC");
    return new Model(name, "en", "UTC");
  }
}
