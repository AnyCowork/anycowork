import { useEffect, useState } from "react";

import { SettingsModal } from "@/components/modals/SettingsModal";
import { CoverImageModal } from "@/components/modals/CoverImageModal";

export const ModalProvider = () => {
  const [isMounted, setIsMounted] = useState(false);

  // Hydration fix
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setIsMounted(true);
  }, []);

  if (!isMounted) {
    return null;
  }

  return (
    <>
      <SettingsModal />
      <CoverImageModal />
    </>
  );
};
