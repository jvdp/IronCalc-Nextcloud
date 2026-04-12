import { useCallback, useEffect, useState } from "react";
import {
  buildFileUrl,
  getLocation,
  type Location,
  locationFromPath,
} from "./utils";

type AsyncResult<T> = { ok: true; data: T } | { ok: false };

export function useAsync<T, Args extends unknown[]>(
  fn: (...args: Args) => Promise<T>,
  deps: unknown[],
): {
  run: (...args: Args) => Promise<AsyncResult<T>>;
  data: T | undefined;
  pending: boolean;
  error: string | null;
} {
  const [data, setData] = useState<T | undefined>(undefined);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: deps are caller-provided
  const stableFn = useCallback(fn, deps);

  const run = useCallback(
    async (...args: Args): Promise<AsyncResult<T>> => {
      setPending(true);
      setError(null);
      try {
        const result = await stableFn(...args);
        setData(result);
        return { ok: true, data: result };
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        return { ok: false };
      } finally {
        setPending(false);
      }
    },
    [stableFn],
  );

  return { run, data, pending, error };
}

export function useLocationState(): {
  location: Location | undefined;
  updateLocation: (newPath: string) => void;
} {
  const [location, setLocation] = useState(getLocation);

  useEffect(() => {
    const sync = () => setLocation(getLocation());
    window.addEventListener("popstate", sync);
    return () => window.removeEventListener("popstate", sync);
  }, []);

  const updateLocation = useCallback((newPath: string) => {
    history.pushState(null, "", buildFileUrl(newPath));
    setLocation(locationFromPath(newPath));
  }, []);

  return { location, updateLocation };
}
