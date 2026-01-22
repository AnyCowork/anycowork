import { useState, useEffect } from "react";

declare global {
    interface Window {
        nativeTitleBar?: boolean;
    }
}

export function useIsDesktopApp() {
    const [isDesktop, setIsDesktop] = useState(false);

    useEffect(() => {
        const checkDesktop = () => {
            // Check for Tauri
            // @ts-ignore
            const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;

            // Check for custom User Agent
            const isCustomUA = typeof navigator !== 'undefined' &&
                navigator.userAgent.includes('AnyCoworkDesktop');

            if (isTauri || isCustomUA) {
                setIsDesktop(true);
                return true;
            }
            return false;
        };

        // Check immediately
        checkDesktop();

        // One-time check after a short delay to catch any late initialization if needed
        const timeout = setTimeout(checkDesktop, 100);

        return () => {
            clearTimeout(timeout);
        };
    }, []);

    return isDesktop;
}
