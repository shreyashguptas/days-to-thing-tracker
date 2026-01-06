"use client";

import { useEffect, useState } from "react";

export function useCountdown(updateIntervalMs: number = 60000) {
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    const interval = setInterval(() => {
      setNow(new Date());
    }, updateIntervalMs);

    return () => clearInterval(interval);
  }, [updateIntervalMs]);

  return now;
}
