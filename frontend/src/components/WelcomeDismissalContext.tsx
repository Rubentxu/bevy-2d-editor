import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";

/**
 * Phase C T3.3 — Shared Welcome visibility context.
 *
 * Provides a context that tracks whether the Welcome overlay should be visible.
 * OnboardingBanner reads from this context to implement mutual exclusion (spec S5):
 * the banner MUST NOT be visible when the Welcome overlay is visible.
 *
 * The context uses an "isChecking" state to prevent both surfaces from rendering
 * during the async OPFS hydration window:
 *   - WelcomeOverlay gates on `isChecking` — stays invisible until OPFS resolves.
 *   - OnboardingBanner gates on `!welcomeVisible` — stays invisible until
 *     WelcomeOverlay's async effect reports the Welcome state.
 *
 * Usage:
 *   <WelcomeDismissalProvider>
 *     <WelcomeOverlay />
 *     <OnboardingBanner />
 *   </WelcomeDismissalProvider>
 *
 * The context is internal to these two components — App.tsx doesn't need changes.
 */

interface ShouldShowResult {
  shouldShow: boolean;
  permanentDismissal: boolean;
}

interface WelcomeDismissalContextValue {
  /** True when Welcome overlay should be shown to the user (set after async OPFS check) */
  welcomeVisible: boolean;
  /** True while WelcomeOverlay is checking OPFS — both surfaces stay hidden */
  isChecking: boolean;
  /** Called by WelcomeOverlay after its OPFS check to report the result */
  reportWelcomeShouldShow: (result: ShouldShowResult) => void;
}

const WelcomeDismissalContext = createContext<WelcomeDismissalContextValue>({
  welcomeVisible: false,
  isChecking: true,
  reportWelcomeShouldShow: () => {},
});

export function WelcomeDismissalProvider({
  children,
}: {
  children: ReactNode;
}) {
  // welcomeVisible: false initially, set to true by WelcomeOverlay's async effect
  //   when Welcome should be shown (first visit, or no prior "Don't show again").
  // isChecking: true while OPFS is being read — both surfaces stay invisible.
  // WelcomeOverlay sets it to false after the async OPFS check.
  const [welcomeVisible, setWelcomeVisible] = useState(false);
  const [isChecking, setIsChecking] = useState(true);

  const reportWelcomeShouldShow = useCallback((result: ShouldShowResult) => {
    setIsChecking(false);
    // Show Welcome only if it should show AND hasn't been permanently dismissed
    setWelcomeVisible(result.shouldShow && !result.permanentDismissal);
  }, []);

  return (
    <WelcomeDismissalContext.Provider
      value={{ welcomeVisible, isChecking, reportWelcomeShouldShow }}
    >
      {children}
    </WelcomeDismissalContext.Provider>
  );
}

/** Hook for child components to access WelcomeDismissal context */
export function useWelcomeDismissal() {
  return useContext(WelcomeDismissalContext);
}
