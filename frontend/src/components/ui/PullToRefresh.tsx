import { ReactNode, useEffect, useRef, useState } from "react";
import { RefreshCw } from "lucide-react";

/** True when running as an installed home-screen app, where there is no browser chrome. */
function isStandalone(): boolean {
  return (
    window.matchMedia("(display-mode: standalone)").matches ||
    ("standalone" in navigator &&
      (navigator as Navigator & { standalone?: boolean }).standalone === true)
  );
}

const TRIGGER_AT = 70; // px of pull needed to trigger
const MAX_PULL = 110;

/**
 * Native-style pull-to-refresh for the installed app. Browser tabs already have
 * a reload button and Safari's own gesture, so this only arms in standalone
 * mode. A full reload also picks up freshly deployed frontend code, which iOS
 * home-screen apps otherwise cache very aggressively.
 */
export function PullToRefresh({ children }: { children: ReactNode }) {
  const [pull, setPull] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const startY = useRef<number | null>(null);
  const pullRef = useRef(0);

  useEffect(() => {
    if (!isStandalone()) return;

    const onTouchStart = (e: TouchEvent) => {
      // Only arm when the page is scrolled to the very top.
      if (window.scrollY <= 0) {
        startY.current = e.touches[0].clientY;
      } else {
        startY.current = null;
      }
    };

    const onTouchMove = (e: TouchEvent) => {
      if (startY.current === null) return;
      const dy = e.touches[0].clientY - startY.current;
      if (dy <= 0) {
        pullRef.current = 0;
        setPull(0);
        return;
      }
      // Resist the pull so it feels physical rather than 1:1.
      const next = Math.min(dy * 0.45, MAX_PULL);
      pullRef.current = next;
      setPull(next);
    };

    const onTouchEnd = () => {
      if (startY.current === null) return;
      startY.current = null;
      if (pullRef.current >= TRIGGER_AT) {
        setRefreshing(true);
        window.location.reload();
      } else {
        pullRef.current = 0;
        setPull(0);
      }
    };

    window.addEventListener("touchstart", onTouchStart, { passive: true });
    window.addEventListener("touchmove", onTouchMove, { passive: true });
    window.addEventListener("touchend", onTouchEnd, { passive: true });
    return () => {
      window.removeEventListener("touchstart", onTouchStart);
      window.removeEventListener("touchmove", onTouchMove);
      window.removeEventListener("touchend", onTouchEnd);
    };
  }, []);

  const armed = pull >= TRIGGER_AT;
  const visible = pull > 8 || refreshing;

  return (
    <>
      {visible && (
        <div
          className="pointer-events-none fixed inset-x-0 z-50 flex justify-center"
          style={{ top: `calc(env(safe-area-inset-top) + ${Math.min(pull, MAX_PULL) * 0.5}px)` }}
        >
          <div
            className={`rounded-full border border-sand-300 bg-sand-50 p-2 shadow-md transition-colors dark:border-charcoal-700 dark:bg-charcoal-800 ${
              armed || refreshing
                ? "text-sage-600 dark:text-sage-400"
                : "text-charcoal-400 dark:text-charcoal-500"
            }`}
          >
            <RefreshCw
              size={18}
              className={refreshing ? "animate-spin" : ""}
              style={refreshing ? undefined : { transform: `rotate(${pull * 2.5}deg)` }}
            />
          </div>
        </div>
      )}
      {children}
    </>
  );
}
