import { useEffect, useRef } from "preact/hooks";

/**
 * Debounced auto-save. Calls `saveFn` after `delay` ms of inactivity
 * when `dirty` is true. Resets timer on each render where dirty=true.
 */
export function useAutoSave(
  saveFn: () => void,
  dirty: boolean,
  delay = 5000,
) {
  const timerRef = useRef<number | null>(null);
  const saveFnRef = useRef(saveFn);
  saveFnRef.current = saveFn;

  useEffect(() => {
    if (!dirty) {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
      return;
    }
    // Reset timer
    if (timerRef.current !== null) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      saveFnRef.current();
      timerRef.current = null;
    }, delay) as unknown as number;

    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    };
  }, [dirty, delay]);
}
